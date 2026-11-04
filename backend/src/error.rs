//! Errores de la aplicación y su conversión a respuestas HTTP.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Error unificado de la capa HTTP.
///
/// Todos los endpoints devuelven el mismo sobre JSON:
/// `{"error": "<tipo>", "message": "<detalle>"}`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Recurso no encontrado (404).
    #[error("recurso no encontrado: {0}")]
    NotFound(String),
    /// Datos de entrada inválidos (400).
    #[error("datos inválidos: {0}")]
    Validation(String),
    /// Conflicto con el estado actual del recurso (409).
    #[error("conflicto: {0}")]
    Conflict(String),
    /// Método HTTP no permitido en la ruta (405).
    #[error("método no permitido: {0}")]
    MethodNotAllowed(String),
    /// El cuerpo es sintácticamente válido pero no procesable (422).
    ///
    /// Se usa, p. ej., cuando la página descargada no contiene una receta
    /// estructurada (`@type: Recipe`).
    #[error("no procesable: {0}")]
    UnprocessableEntity(String),
    /// El origen remoto falló al ser consultado (502).
    #[error("fallo del origen remoto: {0}")]
    BadGateway(String),
    /// Error de base de datos (500, detalle no expuesto al cliente).
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Error interno inesperado (500, detalle no expuesto al cliente).
    #[error("error interno: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        AppError::Internal(anyhow::Error::new(error))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, kind, message) = match &self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message.clone()),
            AppError::Validation(message) => {
                (StatusCode::BAD_REQUEST, "validation", message.clone())
            }
            AppError::Conflict(message) => (StatusCode::CONFLICT, "conflict", message.clone()),
            AppError::MethodNotAllowed(message) => (
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                message.clone(),
            ),
            AppError::UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unprocessable_entity",
                message.clone(),
            ),
            AppError::BadGateway(message) => {
                (StatusCode::BAD_GATEWAY, "bad_gateway", message.clone())
            }
            AppError::Database(error) => {
                tracing::error!(error = %error, "error de base de datos");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    INTERNAL_MESSAGE.to_string(),
                )
            }
            AppError::Internal(error) => {
                tracing::error!(error = %error, "error interno");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    INTERNAL_MESSAGE.to_string(),
                )
            }
        };

        (status, Json(json!({ "error": kind, "message": message }))).into_response()
    }
}

const INTERNAL_MESSAGE: &str = "error interno del servidor";

/// Alias de `Result` con [`AppError`] como tipo de error.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_404() {
        let response = AppError::NotFound("receta".into()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn validation_maps_to_400() {
        let response = AppError::Validation("campo requerido".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn method_not_allowed_maps_to_405() {
        let response = AppError::MethodNotAllowed("POST".into()).into_response();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn conflict_maps_to_409() {
        let response = AppError::Conflict("receta en uso".into()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn unprocessable_entity_maps_to_422() {
        let response = AppError::UnprocessableEntity("HTML sin receta".into()).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn bad_gateway_maps_to_502() {
        let response = AppError::BadGateway("origen no responde".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn internal_does_not_leak_details() {
        let response = AppError::Internal(anyhow::anyhow!("secreto interno")).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("leer body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json del body");

        assert_eq!(
            json,
            serde_json::json!({
                "error": "internal",
                "message": "error interno del servidor"
            })
        );
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert!(
            !text.contains("secreto interno"),
            "el detalle interno no debe filtrarse: {text}"
        );
    }
}
