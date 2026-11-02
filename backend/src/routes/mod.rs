//! Definición de las rutas HTTP de la API.

pub mod agent;
pub mod health;
pub mod meal_plan;
pub mod openapi;
pub mod recipes;
pub mod shopping_list;
pub mod validation;

use axum::Router;

use crate::state::AppState;

/// Construye el router de la API, aún sin el estado inyectado.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(recipes::router())
        .merge(meal_plan::router())
        .merge(shopping_list::router())
        .merge(agent::router())
        .merge(openapi::router())
}
