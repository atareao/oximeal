//! Endpoint de salud del servicio.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::db;
use crate::state::AppState;

/// Router del health check.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/health", get(health))
}

/// `GET /api/health` — comprueba que el servicio y la base de datos responden.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "system",
    responses(
        (status = 200, description = "Servicio y base de datos sanos"),
        (status = 503, description = "Base de datos no disponible"),
    )
)]
pub async fn health(State(state): State<AppState>) -> Response {
    match db::ping(&state.pool).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok", "db": "ok" }))).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "health check: la base de datos no responde");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "degraded", "db": "error" })),
            )
                .into_response()
        }
    }
}
