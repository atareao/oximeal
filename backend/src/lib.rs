//! Crate de OxiMeal Backend: núcleo del servidor y dominio puro.

pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod models;
pub mod repo;
pub mod routes;
pub mod state;
pub mod static_files;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Construye el router completo de la aplicación con el estado inyectado.
///
/// Aplica trazabilidad de peticiones y CORS permisivo (despliegue self-hosted).
pub fn app(state: AppState) -> Router {
    // Servicio self-hosted: se permite cualquier origen/método/cabecera.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(routes::router())
        // El fallback del frontend debe fusionarse al final: solo actúa cuando
        // ninguna ruta de la API ha casado.
        //
        // ADVERTENCIA: `routes::router()` NO debe definir un `fallback()` propio.
        // Fusionar dos routers que aportan un `fallback` hace que axum entre en
        // pánico ("Cannot merge two `Router`s that both have a fallback"). El
        // fallback único del árbol es el de `static_files::router()`.
        .merge(static_files::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
