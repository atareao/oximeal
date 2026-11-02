//! Errores de la capa de autenticación y su conversión a respuestas HTTP.
//!
//! `AuthError` sigue el mismo sobre JSON que [`crate::error::AppError`]
//! (`{"error": "<tipo>", "message": "<detalle>"}`). El caso 401 añade la
//! cabecera `WWW-Authenticate: Bearer`, exigida por el contrato de la API.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Error de la capa de autenticación.
///
/// Nunca incluye tokens ni secretos en el mensaje: los detalles de un fallo
/// aguas arriba se registran en `tracing` y se sustituyen por un mensaje
/// genérico de cara al cliente.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// No hay credencial válida (401 + `WWW-Authenticate: Bearer`).
    #[error("no autenticado")]
    Unauthorized,
    /// Hay credencial válida pero carece de permiso / grupo (403).
    #[error("acceso prohibido")]
    Forbidden,
    /// Petición malformada (400). El detalle sí se expone (no contiene secretos).
    #[error("petición inválida: {0}")]
    BadRequest(String),
    /// Fallo del proveedor OIDC o de la red (500, detalle no expuesto).
    #[error("error del proveedor OIDC: {0}")]
    Upstream(String),
    /// Fallo del almacén de sesiones (500, detalle no expuesto).
    #[error("error del almacén de sesiones: {0}")]
    Store(String),
    /// Error interno inesperado (500, detalle no expuesto).
    #[error("error interno de autenticación: {0}")]
    Internal(String),
}

impl AuthError {
    /// Construye un error de almacén a partir de un `sqlx::Error`.
    pub fn store(error: impl std::fmt::Display) -> Self {
        AuthError::Store(error.to_string())
    }
}

const INTERNAL_MESSAGE: &str = "error interno del servidor";

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, kind, message) = match &self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AuthError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, "validation", message.clone())
            }
            AuthError::Upstream(error) => {
                tracing::error!(error = %error, "fallo del proveedor OIDC");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    INTERNAL_MESSAGE.to_string(),
                )
            }
            AuthError::Store(error) => {
                tracing::error!(error = %error, "fallo del almacén de sesiones");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    INTERNAL_MESSAGE.to_string(),
                )
            }
            AuthError::Internal(error) => {
                tracing::error!(error = %error, "error interno de autenticación");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    INTERNAL_MESSAGE.to_string(),
                )
            }
        };

        let mut response =
            (status, Json(json!({ "error": kind, "message": message }))).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(error: sqlx::Error) -> Self {
        AuthError::store(error)
    }
}

/// Alias de `Result` con [`AuthError`] como tipo de error.
pub type AuthResult<T> = Result<T, AuthError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_maps_to_401_with_www_authenticate() {
        let response = AuthError::Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response
                .headers()
                .get(header::WWW_AUTHENTICATE)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer")
        );
    }

    #[test]
    fn forbidden_maps_to_403() {
        let response = AuthError::Forbidden.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn bad_request_maps_to_400() {
        let response = AuthError::BadRequest("state".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn upstream_does_not_leak_details() {
        let response = AuthError::Upstream("client_secret=abc".into()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
