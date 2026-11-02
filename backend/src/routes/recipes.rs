//! Endpoints REST de recetas.

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::recipe::{Recipe, RecipeInput, RecipeListResponse, RecipeQuery};
use crate::repo;
use crate::routes::validation;
use crate::services;
use crate::state::AppState;

/// Cuerpo de `POST /api/recipes/import`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportRecipeRequest {
    /// URL `http(s)` de la página que publica el JSON-LD `Recipe`.
    pub url: String,
}

/// Router de recetas de lectura (permiso `read:recipes`).
pub fn read_router() -> Router<AppState> {
    Router::new()
        .route("/api/recipes", get(list_recipes))
        .route("/api/recipes/{id}", get(get_recipe))
}

/// Router de recetas de escritura (permiso `write:recipes`).
pub fn write_router() -> Router<AppState> {
    Router::new()
        .route("/api/recipes", post(create_recipe))
        .route("/api/recipes/import", post(import_recipe))
        .route(
            "/api/recipes/{id}",
            put(update_recipe).delete(delete_recipe),
        )
}

/// `POST /api/recipes/import` — importa una receta desde una URL (Schema.org).
///
/// Cuerpo `{ "url": "<http(s)>" }`. Descarga el HTML, extrae el JSON-LD
/// `@type: Recipe` y persiste la receta de forma idempotente por `source_url`
/// (201 si se crea, 200 si ya existía).
#[utoipa::path(
    post,
    path = "/api/recipes/import",
    tag = "recipes",
    request_body = ImportRecipeRequest,
    responses(
        (status = 201, description = "Receta importada", body = Recipe),
        (status = 200, description = "Receta ya importada (idempotente)", body = Recipe),
        (status = 400, description = "URL inválida o prohibida (SSRF)"),
        (status = 422, description = "HTML sin receta estructurada"),
        (status = 502, description = "Fallo del origen remoto"),
    )
)]
pub async fn import_recipe(
    State(state): State<AppState>,
    payload: Result<Json<ImportRecipeRequest>, JsonRejection>,
) -> AppResult<(StatusCode, Json<Recipe>)> {
    // Un cuerpo JSON ausente o malformado se traduce al sobre de error estándar
    // de la app (400) en lugar del rechazo por defecto de axum.
    let Json(body) = payload.map_err(|rejection| {
        AppError::Validation(format!("cuerpo JSON inválido: {}", rejection.body_text()))
    })?;

    let outcome = services::recipe_import::import_from_url(&state, &body.url).await?;

    let status = if outcome.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    Ok((status, Json(outcome.recipe)))
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
    validation::validate_recipe(&input)?;

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

/// `PUT /api/recipes/{id}` — reemplaza una receta completa.
#[utoipa::path(
    put,
    path = "/api/recipes/{id}",
    tag = "recipes",
    params(("id" = String, Path, description = "Identificador de la receta")),
    request_body = RecipeInput,
    responses(
        (status = 200, description = "Receta actualizada", body = Recipe),
        (status = 400, description = "Datos inválidos"),
        (status = 404, description = "Receta no encontrada"),
    )
)]
pub async fn update_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<RecipeInput>,
) -> AppResult<Json<Recipe>> {
    validation::validate_recipe(&input)?;

    let recipe = repo::recipes::update(&state.pool, &id, &input).await?;

    Ok(Json(recipe))
}

/// `DELETE /api/recipes/{id}` — borra una receta.
#[utoipa::path(
    delete,
    path = "/api/recipes/{id}",
    tag = "recipes",
    params(("id" = String, Path, description = "Identificador de la receta")),
    responses(
        (status = 204, description = "Receta eliminada"),
        (status = 404, description = "Receta no encontrada"),
        (status = 409, description = "La receta está en uso en el plan semanal"),
    )
)]
pub async fn delete_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    repo::recipes::delete(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// La validación del cuerpo vive en `routes::validation::validate_recipe`,
// compartida con la importación desde URL.
