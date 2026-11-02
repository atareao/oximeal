//! Endpoints REST de recetas.

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::recipe::{Recipe, RecipeInput, RecipeListResponse, RecipeQuery};
use crate::repo;
use crate::routes::validation;
use crate::state::AppState;

/// Router de recetas de lectura (permiso `read:recipes`).
pub fn read_router() -> Router<AppState> {
    Router::new()
        .route("/api/recipes", get(list_recipes))
        .route("/api/recipes/{id}", get(get_recipe))
}

/// Router de recetas de escritura (permiso `write:recipes`).
pub fn write_router() -> Router<AppState> {
    Router::new().route("/api/recipes", post(create_recipe))
}

/// `GET /api/recipes` — listado con búsqueda (FTS5/LIKE), filtro y paginación.
#[utoipa::path(
    get,
    path = "/api/recipes",
    tag = "recipes",
    params(
        ("q" = Option<String>, Query, description = "Texto de búsqueda (FTS5 si >= 3 caracteres)"),
        ("category" = Option<String>, Query, description = "Filtro por categoría"),
        ("page" = Option<u32>, Query, description = "Página (1-indexada)"),
        ("page_size" = Option<u32>, Query, description = "Tamaño de página (máx 100)"),
    ),
    responses(
        (status = 200, description = "Listado paginado de recetas", body = RecipeListResponse),
        (status = 400, description = "Categoría inválida"),
    )
)]
pub async fn list_recipes(
    State(state): State<AppState>,
    Query(query): Query<RecipeQuery>,
) -> AppResult<Json<RecipeListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let category = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(category) = category
        && !validation::is_valid_category(category)
    {
        return Err(AppError::Validation(format!(
            "categoría inválida: {category}"
        )));
    }

    let response =
        repo::recipes::list(&state.pool, query.q.as_deref(), category, page, page_size).await?;

    Ok(Json(response))
}

/// `POST /api/recipes` — alta de receta.
#[utoipa::path(
    post,
    path = "/api/recipes",
    tag = "recipes",
    request_body = RecipeInput,
    responses(
        (status = 201, description = "Receta creada", body = Recipe),
        (status = 400, description = "Datos inválidos"),
    )
)]
pub async fn create_recipe(
    State(state): State<AppState>,
    Json(input): Json<RecipeInput>,
) -> AppResult<impl IntoResponse> {
    validate_recipe(&input)?;

    let id = Uuid::new_v4().to_string();
    let recipe = repo::recipes::create(&state.pool, &id, &input).await?;

    Ok((StatusCode::CREATED, Json(recipe)))
}

/// `GET /api/recipes/{id}` — detalle completo de una receta.
#[utoipa::path(
    get,
    path = "/api/recipes/{id}",
    tag = "recipes",
    params(("id" = String, Path, description = "Identificador de la receta")),
    responses(
        (status = 200, description = "Receta encontrada", body = Recipe),
        (status = 404, description = "Receta no encontrada"),
    )
)]
pub async fn get_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Recipe>> {
    let recipe = repo::recipes::get(&state.pool, &id).await?;
    Ok(Json(recipe))
}

/// Valida el cuerpo de creación de receta. No persiste nada si falla.
fn validate_recipe(input: &RecipeInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "el nombre no puede estar vacío".into(),
        ));
    }
    if !validation::is_valid_category(&input.category) {
        return Err(AppError::Validation(format!(
            "categoría inválida: {}",
            input.category
        )));
    }
    if input.ingredients.is_empty() {
        return Err(AppError::Validation(
            "la receta debe tener al menos un ingrediente".into(),
        ));
    }
    for ingredient in &input.ingredients {
        if ingredient.name.trim().is_empty() {
            return Err(AppError::Validation(
                "el nombre del ingrediente no puede estar vacío".into(),
            ));
        }
        if ingredient.amount < 0.0 {
            return Err(AppError::Validation(
                "el amount del ingrediente debe ser >= 0".into(),
            ));
        }
    }
    if let Some(servings) = input.servings
        && servings < 1
    {
        return Err(AppError::Validation("servings debe ser >= 1".into()));
    }
    if let Some(prep) = input.prep_time_minutes
        && prep < 0
    {
        return Err(AppError::Validation(
            "prep_time_minutes debe ser >= 0".into(),
        ));
    }

    Ok(())
}
