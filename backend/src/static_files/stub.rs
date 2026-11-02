//! Implementación sin UI empaquetada: 404 con pista en rutas no-API.

use axum::http::Uri;
use axum::response::Response;

use super::{api_not_found, is_api_path, ui_not_packaged};

/// Responde 404: JSON de API para `/api/*`, pista de UI para el resto.
pub async fn serve(uri: Uri) -> Response {
    let path = uri.path();

    if is_api_path(path) {
        return api_not_found(path);
    }

    ui_not_packaged()
}
