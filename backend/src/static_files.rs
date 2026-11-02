//! Servido del frontend compilado y *fallback* SPA.
//!
//! Este módulo aporta el `fallback` del router raíz: cualquier petición que no
//! case con una ruta registrada (la API bajo `/api`) llega aquí.
//!
//! - Con la feature `embed-ui` sirve los ficheros de `frontend/dist`
//!   incrustados con `rust-embed` y aplica el *fallback* SPA devolviendo
//!   `index.html` para rutas de cliente desconocidas.
//! - Sin la feature responde 404 con una pista sobre cómo habilitar la UI.
//!
//! En ningún caso se sirve `index.html` bajo `/api/*` ni bajo `/auth/*`: esas
//! rutas devuelven siempre el 404 JSON de la API cuando no existe una ruta
//! registrada. Con OIDC desactivado, `/auth/*` no se monta y el frontend
//! necesita ese 404 JSON para concluir que la autenticación está apagada.

use axum::Router;
use axum::response::{IntoResponse, Response};

#[cfg(not(feature = "embed-ui"))]
use axum::Json;
#[cfg(not(feature = "embed-ui"))]
use axum::http::StatusCode;

use crate::error::AppError;
use crate::state::AppState;

#[cfg(feature = "embed-ui")]
mod embedded;
#[cfg(not(feature = "embed-ui"))]
mod stub;

#[cfg(feature = "embed-ui")]
use embedded as ui;
#[cfg(not(feature = "embed-ui"))]
use stub as ui;

/// Construye el router encargado del frontend.
///
/// Solo define un `fallback`; debe fusionarse **después** de `routes::router()`
/// para que las rutas de la API tengan prioridad.
pub fn router() -> Router<AppState> {
    Router::new().fallback(ui::serve)
}

/// `true` si la ruta pertenece a la API (`/api`, `/api/...`, `/auth` o
/// `/auth/...`).
///
/// Clasifica sobre la **clave normalizada**, no sobre el path crudo, para que
/// variantes mal formadas como `//api/health`, `/./api/health`, `//auth/me` o
/// `/./auth/login` se detecten igualmente como API y nunca caigan al *fallback*
/// SPA. `/auth/*` se incluye porque, con OIDC desactivado, esas rutas no se
/// montan y debe devolverse el 404 JSON de la API (no `index.html`).
fn is_api_path(path: &str) -> bool {
    let key = normalize(path);
    key == "api" || key.starts_with("api/") || key == "auth" || key.starts_with("auth/")
}

/// Normaliza la ruta de la petición a una clave canónica del frontend.
///
/// - `/` (o vacío) → `index.html`.
/// - Colapsa segmentos vacíos (`//`), `.` y `..`.
///
/// Es defensa en profundidad: aunque `rust-embed` nunca resuelve fuera del
/// árbol incrustado, evitamos construir claves con `..`. No se decodifica
/// *percent-encoding*: `%2e%2e` es un segmento literal, no un `..`, de modo que
/// tampoco puede escapar del árbol.
fn normalize(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    if segments.is_empty() {
        "index.html".to_string()
    } else {
        segments.join("/")
    }
}

/// Respuesta 404 JSON coherente con el resto de la API.
fn api_not_found(path: &str) -> Response {
    AppError::NotFound(path.to_string()).into_response()
}

/// 404 JSON indicando que la UI no va empaquetada en este binario.
#[cfg(not(feature = "embed-ui"))]
fn ui_not_packaged() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "UI no empaquetada: compila con --features embed-ui"
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::is_api_path;

    #[test]
    fn api_paths_are_classified_as_api() {
        for path in [
            "api",
            "/api",
            "/api/health",
            "//api/health",
            "/./api/health",
        ] {
            assert!(is_api_path(path), "{path} debería clasificarse como API");
        }
    }

    #[test]
    fn auth_paths_are_classified_as_api() {
        for path in [
            "auth",
            "/auth",
            "/auth/me",
            "/auth/login",
            "//auth/me",
            "/./auth/login",
        ] {
            assert!(is_api_path(path), "{path} debería clasificarse como API");
        }
    }

    #[test]
    fn client_paths_are_not_classified_as_api() {
        for path in [
            "/shopping",
            "/recetas/pasta.v2",
            "/assets/index-abc.js",
            "/authentication",
            "/authors",
            "/authz",
            "/",
            "",
        ] {
            assert!(
                !is_api_path(path),
                "{path} NO debería clasificarse como API"
            );
        }
    }
}
