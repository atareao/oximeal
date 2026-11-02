//! Helpers compartidos por los tests de integración.
//!
//! Crean una base de datos SQLite temporal por test, aplican las migraciones
//! y construyen un [`AppState`] listo para usar con el router de `app()`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oximeal_backend::config::Config;
use oximeal_backend::db;
use oximeal_backend::state::AppState;
use std::sync::Arc;
use tower::ServiceExt;

/// Crea una base de datos SQLite temporal con las migraciones aplicadas.
///
/// El `TempDir` se filtra deliberadamente (`mem::forget`) para que el fichero
/// no se borre mientras el pool sigue abierto; es un proceso de test efímero.
pub async fn test_state() -> AppState {
    let dir = tempfile::tempdir().expect("crear tempdir");
    let db_path = dir.path().join("test.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let pool = db::init_pool(&database_url)
        .await
        .expect("inicializar pool de test");

    // Mantener vivo el directorio temporal durante la vida del proceso de test.
    std::mem::forget(dir);

    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".to_string(),
    };

    AppState {
        pool,
        config: Arc::new(config),
    }
}

/// Envía una petición al router completo y devuelve el estado HTTP y el JSON.
///
/// Si `body` es `Some`, se serializa como JSON y se fija la cabecera
/// `content-type`. Si la respuesta no tiene cuerpo (p. ej. 204), devuelve
/// `serde_json::Value::Null`.
#[allow(dead_code)]
pub async fn request(
    state: AppState,
    method: &str,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo JSON"))
        }
        None => Body::empty(),
    };
    let request = builder.body(request_body).expect("construir request");

    let response = oximeal_backend::app(state)
        .oneshot(request)
        .await
        .expect("ejecutar request");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes();
    let value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };

    (status, value)
}
