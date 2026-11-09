//! Endpoints REST de la lista de la compra.

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use chrono::{Datelike, Duration, Local, NaiveDate};

use crate::domain::category::CATEGORY_ORDER;
use crate::error::{AppError, AppResult};
use crate::models::shopping_list::{
    ConsolidateInput, PlanToShoppingListInput, RecipeToShoppingListInput, ShoppingItem,
    ShoppingItemInput, ShoppingItemPatch, ShoppingListGroup,
};
use crate::repo;
use crate::routes::validation;
use crate::state::AppState;

/// Router de la lista de la compra de lectura (permiso `read:shopping`).
pub fn read_router() -> Router<AppState> {
    Router::new().route("/api/shopping-list", get(list_shopping_list))
}

/// Router de la lista de la compra de escritura (permiso `write:shopping`).
pub fn write_router() -> Router<AppState> {
    Router::new()
        .route("/api/shopping-list", post(create_item))
        .route(
            "/api/shopping-list/completed",
            axum::routing::delete(delete_completed),
        )
        .route("/api/shopping-list/consolidate", post(consolidate))
        .route("/api/shopping-list/from-recipe", post(from_recipe))
        .route("/api/shopping-list/from-plan", post(from_plan))
        .route("/api/shopping-list/{id}", patch(patch_item))
}

/// `GET /api/shopping-list` — lista agrupada por categoría de pasillo.
#[utoipa::path(
    get,
    path = "/api/shopping-list",
    tag = "shopping-list",
    responses(
        (status = 200, description = "Lista agrupada por categoría", body = [ShoppingListGroup]),
    )
)]
pub async fn list_shopping_list(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ShoppingListGroup>>> {
    let groups = repo::shopping_list::list_grouped(&state.pool).await?;
    Ok(Json(groups))
}

/// `POST /api/shopping-list` — alta manual de un ítem.
///
/// Si el cuerpo trae un `client_id` que ya existe, devuelve `200` con el ítem
/// existente (alta idempotente); en caso contrario crea el ítem y responde `201`.
#[utoipa::path(
    post,
    path = "/api/shopping-list",
    tag = "shopping-list",
    request_body = ShoppingItemInput,
    responses(
        (status = 201, description = "Ítem creado", body = ShoppingItem),
        (status = 200, description = "Ítem existente (alta idempotente)", body = ShoppingItem),
        (status = 400, description = "Datos inválidos"),
    )
)]
pub async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<ShoppingItemInput>,
) -> AppResult<impl IntoResponse> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "el nombre no puede estar vacío".into(),
        ));
    }
    if input.amount < 0.0 {
        return Err(AppError::Validation("amount debe ser >= 0".into()));
    }

    let outcome = repo::shopping_list::create_item(&state.pool, &input).await?;
    let (status, item) = match outcome {
        repo::shopping_list::CreateOutcome::Created(item) => (StatusCode::CREATED, item),
        repo::shopping_list::CreateOutcome::Existing(item) => (StatusCode::OK, item),
    };
    Ok((status, Json(item)))
}

/// `PATCH /api/shopping-list/{id}` — alterna `checked` o actualiza `amount`.
#[utoipa::path(
    patch,
    path = "/api/shopping-list/{id}",
    tag = "shopping-list",
    params(("id" = String, Path, description = "Identificador del ítem")),
    request_body = ShoppingItemPatch,
    responses(
        (status = 200, description = "Ítem actualizado", body = ShoppingItem),
        (status = 400, description = "Cuerpo vacío o inválido"),
        (status = 404, description = "Ítem no encontrado"),
    )
)]
pub async fn patch_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(patch): Json<ShoppingItemPatch>,
) -> AppResult<Json<ShoppingItem>> {
    if patch.checked.is_none() && patch.amount.is_none() && patch.category.is_none() {
        return Err(AppError::Validation(
            "debe indicar checked, amount o category".into(),
        ));
    }
    if let Some(amount) = patch.amount
        && amount < 0.0
    {
        return Err(AppError::Validation("amount debe ser >= 0".into()));
    }
    if let Some(category) = patch.category.as_deref()
        && !CATEGORY_ORDER.contains(&category)
    {
        return Err(AppError::Validation(format!(
            "categoría inválida: {category}"
        )));
    }

    let item = repo::shopping_list::patch_item(&state.pool, &id, &patch).await?;
    Ok(Json(item))
}

/// `POST /api/shopping-list/consolidate` — regenera la lista desde el plan.
///
/// **Semántica**: la lista de la compra es una **única lista semanal global**.
/// Este endpoint recalcula las líneas derivadas del plan (`source_plan_id` no
/// nulo) a partir de las entradas del plan comprendidas en el rango —o en la
/// semana ISO actual si no se indica— y las reemplaza *en su totalidad*,
/// conservando los ítems manuales. El rango **no** acota el borrado: solo decide
/// qué entradas se leen para consolidar.
#[utoipa::path(
    post,
    path = "/api/shopping-list/consolidate",
    tag = "shopping-list",
    request_body = ConsolidateInput,
    responses(
        (status = 200, description = "Lista consolidada resultante", body = [ShoppingListGroup]),
        (status = 400, description = "Rango de fechas inválido"),
    )
)]
pub async fn consolidate(
    State(state): State<AppState>,
    payload: Option<Json<ConsolidateInput>>,
) -> AppResult<Json<Vec<ShoppingListGroup>>> {
    let input = payload.map(|Json(input)| input).unwrap_or_default();
    let (start, end) = resolve_range(&input)?;

    let groups =
        repo::shopping_list::consolidate(&state.pool, &start.to_string(), &end.to_string()).await?;
    Ok(Json(groups))
}

/// `POST /api/shopping-list/from-recipe` — vuelca una receta a la lista.
///
/// **Semántica**: registro declarativo. Se registra (o actualiza) la receta y sus
/// raciones en `shopping_recipe_sources` y se reconstruyen **todas** las líneas
/// derivadas de recetas reconsolidando las fuentes registradas. Repetir la
/// petición es idempotente y cambiar `servings` reescala sin acumular. Las líneas
/// del plan (`source_plan_id` no nulo) y los ítems manuales se conservan.
#[utoipa::path(
    post,
    path = "/api/shopping-list/from-recipe",
    tag = "shopping-list",
    request_body = RecipeToShoppingListInput,
    responses(
        (status = 200, description = "Lista resultante del volcado", body = [ShoppingListGroup]),
        (status = 400, description = "Cuerpo inválido o `servings` < 1"),
        (status = 404, description = "Receta no encontrada"),
    )
)]
pub async fn from_recipe(
    State(state): State<AppState>,
    payload: Result<Json<RecipeToShoppingListInput>, JsonRejection>,
) -> AppResult<Json<Vec<ShoppingListGroup>>> {
    // Un cuerpo JSON ausente o malformado (p. ej. `servings` no entero) se
    // traduce al sobre de error estándar de la app (400) en lugar del rechazo
    // por defecto de axum.
    let Json(input) = payload.map_err(|rejection| {
        AppError::Validation(format!("cuerpo JSON inválido: {}", rejection.body_text()))
    })?;

    if input.recipe_id.trim().is_empty() {
        return Err(AppError::Validation(
            "recipe_id no puede estar vacío".into(),
        ));
    }

    let groups =
        repo::shopping_list::from_recipe(&state.pool, input.recipe_id.trim(), input.servings)
            .await?;
    Ok(Json(groups))
}

/// `POST /api/shopping-list/from-plan` — vuelca aditivamente los ingredientes del plan.
///
/// **Semántica**: a diferencia de `consolidate`, este volcado es **aditivo**:
/// fusiona los ingredientes de las entradas del plan en el rango con los ítems
/// existentes por nombre normalizado y unidad canónica. Si coincide, suma la
/// cantidad; si no, inserta un ítem nuevo. Los ítems manuales, los volcados de
/// receta y los del plan se conservan con sus `checked`/`client_id`.
#[utoipa::path(
    post,
    path = "/api/shopping-list/from-plan",
    tag = "shopping-list",
    request_body = PlanToShoppingListInput,
    responses(
        (status = 200, description = "Lista resultante del volcado", body = [ShoppingListGroup]),
        (status = 400, description = "Rango de fechas inválido o cuerpo inválido"),
    )
)]
pub async fn from_plan(
    State(state): State<AppState>,
    payload: Result<Json<PlanToShoppingListInput>, JsonRejection>,
) -> AppResult<Json<Vec<ShoppingListGroup>>> {
    let Json(input) = payload.map_err(|rejection| {
        AppError::Validation(format!("cuerpo JSON inválido: {}", rejection.body_text()))
    })?;

    let start = validation::parse_date(&input.start_date)?;
    let end = validation::parse_date(&input.end_date)?;
    if start > end {
        return Err(AppError::Validation(
            "start_date no puede ser posterior a end_date".into(),
        ));
    }

    let groups =
        repo::shopping_list::from_plan(&state.pool, &start.to_string(), &end.to_string()).await?;
    Ok(Json(groups))
}

/// `DELETE /api/shopping-list/completed` — elimina los ítems marcados.
#[utoipa::path(
    delete,
    path = "/api/shopping-list/completed",
    tag = "shopping-list",
    responses((status = 204, description = "Ítems comprados eliminados")),
)]
pub async fn delete_completed(State(state): State<AppState>) -> AppResult<StatusCode> {
    repo::shopping_list::delete_completed(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Resuelve el rango de consolidación: usa el indicado o la semana ISO actual.
fn resolve_range(input: &ConsolidateInput) -> AppResult<(NaiveDate, NaiveDate)> {
    match (input.start_date.as_deref(), input.end_date.as_deref()) {
        (Some(start), Some(end)) => {
            let start = validation::parse_date(start)?;
            let end = validation::parse_date(end)?;
            if start > end {
                return Err(AppError::Validation(
                    "start_date no puede ser posterior a end_date".into(),
                ));
            }
            Ok((start, end))
        }
        (None, None) => Ok(current_week()),
        _ => Err(AppError::Validation(
            "start_date y end_date deben proporcionarse juntos".into(),
        )),
    }
}

/// Lunes y domingo de la semana actual (hora local).
fn current_week() -> (NaiveDate, NaiveDate) {
    let today = Local::now().date_naive();
    let offset = i64::from(today.weekday().num_days_from_monday());
    let monday = today - Duration::days(offset);
    let sunday = monday + Duration::days(6);
    (monday, sunday)
}
