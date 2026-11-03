//! Endpoints REST de etiquetas de recetas.

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};

use crate::error::{AppError, AppResult};
use crate::models::tag::{Tag, TagInput};
use crate::repo;
use crate::state::AppState;

/// Router de etiquetas de lectura (permiso `read:recipes`).
pub fn read_router() -> Router<AppState> {
    Router::new().route("/api/tags", get(list_tags))
}

/// Router de etiquetas de escritura (permiso `write:recipes`).
pub fn write_router() -> Router<AppState> {
    Router::new()
        .route("/api/tags", post(create_tag))
        .route("/api/tags/{id}", delete(delete_tag))
}

/// `GET /api/tags` — catálogo de etiquetas con su recuento de recetas.
#[utoipa::path(
    get,
    path = "/api/tags",
    tag = "tags",
    responses(
        (status = 200, description = "Listado de etiquetas con recuento", body = [Tag]),
    )
)]
pub async fn list_tags(State(state): State<AppState>) -> AppResult<Json<Vec<Tag>>> {
    let tags = repo::tags::list(&state.pool).await?;
    Ok(Json(tags))
}

/// `POST /api/tags` — crea una etiqueta (idempotente por `slug`).
///
/// Cuerpo `{ "name": "..." }`. Un `name` vacío o solo de espacios devuelve `400`.
/// Si ya existe una etiqueta con el mismo `slug` (normalizado), responde `200`
/// con la existente; si se crea, responde `201`.
#[utoipa::path(
    post,
    path = "/api/tags",
    tag = "tags",
    request_body = TagInput,
    responses(
        (status = 201, description = "Etiqueta creada", body = Tag),
        (status = 200, description = "Etiqueta ya existente (idempotente)", body = Tag),
        (status = 400, description = "Nombre vacío o inválido"),
    )
)]
pub async fn create_tag(
    State(state): State<AppState>,
    Json(input): Json<TagInput>,
) -> AppResult<impl IntoResponse> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "el nombre de la etiqueta no puede estar vacío".into(),
        ));
    }

    let (tag, created) = repo::tags::create(&state.pool, &input.name).await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    Ok((status, Json(tag)))
}

/// `DELETE /api/tags/{id}` — borra una etiqueta y sus asociaciones en cascada.
///
/// Las recetas afectadas permanecen intactas. Devuelve `404` si el id no existe.
#[utoipa::path(
    delete,
    path = "/api/tags/{id}",
    tag = "tags",
    params(("id" = String, Path, description = "Identificador de la etiqueta")),
    responses(
        (status = 204, description = "Etiqueta eliminada"),
        (status = 404, description = "Etiqueta no encontrada"),
    )
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    repo::tags::delete(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
