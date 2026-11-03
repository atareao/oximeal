//! Endpoints REST de la planificación de comidas.

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::meal_plan::{
    MealPlanEntryView, MealPlanInput, MealPlanPatchInput, MealPlanQuery,
};
use crate::repo;
use crate::routes::validation;
use crate::state::AppState;

/// Router de la planificación de lectura (permiso `read:plan`).
pub fn read_router() -> Router<AppState> {
    Router::new().route("/api/meal-plan", get(list_meal_plan))
}

/// Router de la planificación de escritura (permiso `write:plan`).
pub fn write_router() -> Router<AppState> {
    Router::new()
        .route("/api/meal-plan", axum::routing::post(create_meal_plan))
        .route(
            "/api/meal-plan/{id}",
            patch(patch_meal_plan).delete(delete_meal_plan),
        )
}

/// `GET /api/meal-plan` — entradas del plan en un rango de fechas.
#[utoipa::path(
    get,
    path = "/api/meal-plan",
    tag = "meal-plan",
    params(
        ("start_date" = String, Query, description = "Fecha inicial (YYYY-MM-DD)"),
        ("end_date" = String, Query, description = "Fecha final (YYYY-MM-DD)"),
    ),
    responses(
        (status = 200, description = "Entradas del rango ordenadas por fecha y comida", body = [MealPlanEntryView]),
        (status = 400, description = "Rango de fechas inválido"),
    )
)]
pub async fn list_meal_plan(
    State(state): State<AppState>,
    Query(query): Query<MealPlanQuery>,
) -> AppResult<Json<Vec<MealPlanEntryView>>> {
    let start = validation::parse_date(&query.start_date)?;
    let end = validation::parse_date(&query.end_date)?;
    if start > end {
        return Err(AppError::Validation(
            "start_date no puede ser posterior a end_date".into(),
        ));
    }

    // Normaliza el rango a `YYYY-MM-DD` para que coincida con las fechas
    // canónicas almacenadas, aunque lleguen sin rellenar.
    let start = start.format("%Y-%m-%d").to_string();
    let end = end.format("%Y-%m-%d").to_string();

    let entries = repo::meal_plan::list(&state.pool, &start, &end).await?;
    Ok(Json(entries))
}

/// `POST /api/meal-plan` — asigna una receta a un slot del plan.
#[utoipa::path(
    post,
    path = "/api/meal-plan",
    tag = "meal-plan",
    request_body = MealPlanInput,
    responses(
        (status = 201, description = "Entrada creada", body = MealPlanEntryView),
        (status = 400, description = "Datos inválidos o receta inexistente"),
    )
)]
pub async fn create_meal_plan(
    State(state): State<AppState>,
    Json(input): Json<MealPlanInput>,
) -> AppResult<impl IntoResponse> {
    validate_entry(&input)?;

    let id = Uuid::new_v4().to_string();
    let entry = repo::meal_plan::create(&state.pool, &id, &input).await?;

    Ok((StatusCode::CREATED, Json(entry)))
}

/// `PATCH /api/meal-plan/{id}` — actualiza las raciones de una entrada.
///
/// El cuerpo debe incluir `servings >= 1`; se responde `400` si falta o es
/// inválido y `404` si la entrada no existe. No se puede cambiar `recipe_id`,
/// `date`, `meal_type` ni `role`: para eso se elimina y se vuelve a crear.
#[utoipa::path(
    patch,
    path = "/api/meal-plan/{id}",
    tag = "meal-plan",
    params(("id" = String, Path, description = "Identificador de la entrada")),
    request_body = MealPlanPatchInput,
    responses(
        (status = 200, description = "Raciones actualizadas", body = MealPlanEntryView),
        (status = 400, description = "servings ausente o inválido"),
        (status = 404, description = "Entrada no encontrada"),
    )
)]
pub async fn patch_meal_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Result<Json<MealPlanPatchInput>, JsonRejection>,
) -> AppResult<Json<MealPlanEntryView>> {
    // Un cuerpo JSON ausente o malformado se traduce al sobre de error estándar
    // de la app (400) en lugar del rechazo por defecto de axum.
    let Json(input) = payload.map_err(|rejection| {
        AppError::Validation(format!("cuerpo JSON inválido: {}", rejection.body_text()))
    })?;

    let servings = input
        .servings
        .filter(|servings| *servings >= 1)
        .ok_or_else(|| AppError::Validation("servings debe ser >= 1".into()))?;

    let entry = repo::meal_plan::update_servings(&state.pool, &id, servings).await?;
    Ok(Json(entry))
}

/// `DELETE /api/meal-plan/{id}` — elimina una entrada del plan.
#[utoipa::path(
    delete,
    path = "/api/meal-plan/{id}",
    tag = "meal-plan",
    params(("id" = String, Path, description = "Identificador de la entrada")),
    responses(
        (status = 204, description = "Entrada eliminada"),
        (status = 404, description = "Entrada no encontrada"),
    )
)]
pub async fn delete_meal_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    repo::meal_plan::delete(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Valida `meal_type`, `role`, fecha y formato del `recipe_id`.
fn validate_entry(input: &MealPlanInput) -> AppResult<()> {
    if !validation::is_valid_meal_type(&input.meal_type) {
        return Err(AppError::Validation(format!(
            "meal_type inválido: {}",
            input.meal_type
        )));
    }
    if !validation::is_valid_role(&input.role) {
        return Err(AppError::Validation(format!(
            "role inválido: {}",
            input.role
        )));
    }
    if input.recipe_id.trim().is_empty() {
        return Err(AppError::Validation("recipe_id es obligatorio".into()));
    }
    validation::parse_date(&input.date)?;

    if let Some(servings) = input.servings
        && servings < 1
    {
        return Err(AppError::Validation("servings debe ser >= 1".into()));
    }

    Ok(())
}
