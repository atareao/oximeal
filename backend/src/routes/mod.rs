//! Definición de las rutas HTTP de la API.
//!
//! La tabla de permisos por endpoint (bloques 3 y 4) se aplica aquí agrupando cada
//! router de dominio por permiso y envolviéndolo con
//! [`crate::auth::middleware::require_permission`]. Con `OIDC_ENABLED=false`
//! (`state.auth.is_none()`) no se aplica ninguna capa: la API queda abierta.
//!
//! | ruta | permiso |
//! |---|---|
//! | `GET /api/recipes`, `GET /api/recipes/{id}`, `GET /api/tags`, `GET /api/settings` | `read:recipes` |
//! | `POST /api/recipes`, `PUT /api/recipes/{id}`, `DELETE /api/recipes/{id}`, `POST /api/tags`, `DELETE /api/tags/{id}`, `PUT /api/settings` | `write:recipes` |
//! | `GET /api/meal-plan` | `read:plan` |
//! | `POST /api/meal-plan`, `PATCH /api/meal-plan/{id}`, `DELETE /api/meal-plan/{id}` | `write:plan` |
//! | `GET /api/shopping-list` | `read:shopping` |
//! | `POST/PATCH/DELETE /api/shopping-list*` | `write:shopping` |
//! | `GET /api/agent/recent-recipes`, `GET /api/agent/search-recipes` | `agent:read` |
//! | `POST /api/agent/weekly-plan` | `agent:plan` |
//! | `POST /api/agent/shopping-items`, `DELETE /api/agent/shopping/completed` | `agent:shopping` |
//! | `POST /api/agent/recipes` | `agent:write` |
//! | `GET /api/health`, `/api/openapi.json`, `/api/docs*` | público |
//!
//! Los `agent:*` son permisos M2M (Valet); ningún cliente user-delegated los
//! recibe, de modo que un token de usuario sin ellos recibe 403 (bloque 4).

pub mod agent;
pub mod health;
pub mod meal_plan;
pub mod openapi;
pub mod recipes;
pub mod settings;
pub mod shopping_list;
pub mod tags;
pub mod validation;

use axum::Router;

use crate::auth::middleware::require_permission;
use crate::state::AppState;

/// Construye el router de la API, aún sin el estado inyectado.
///
/// Necesita el [`AppState`] para construir las capas de autorización ligadas al
/// servicio de autenticación.
pub fn router(state: &AppState) -> Router<AppState> {
    let public = Router::new()
        .merge(health::router())
        .merge(openapi::router());

    // Modo abierto: sin autenticación no se envuelve nada (passthrough real y
    // sin exigir la capa de sesión, que no existe con OIDC desactivado).
    if state.auth.is_none() {
        return public
            .merge(recipes::read_router())
            .merge(recipes::write_router())
            .merge(tags::read_router())
            .merge(tags::write_router())
            .merge(settings::read_router())
            .merge(settings::write_router())
            .merge(meal_plan::read_router())
            .merge(meal_plan::write_router())
            .merge(shopping_list::read_router())
            .merge(shopping_list::write_router())
            .merge(agent::read_router())
            .merge(agent::plan_router())
            .merge(agent::shopping_router())
            .merge(agent::write_router());
    }

    let protected = Router::new()
        .merge(require_permission(
            recipes::read_router(),
            state,
            "read:recipes",
        ))
        .merge(require_permission(
            recipes::write_router(),
            state,
            "write:recipes",
        ))
        .merge(require_permission(
            tags::read_router(),
            state,
            "read:recipes",
        ))
        .merge(require_permission(
            tags::write_router(),
            state,
            "write:recipes",
        ))
        .merge(require_permission(
            settings::read_router(),
            state,
            "read:recipes",
        ))
        .merge(require_permission(
            settings::write_router(),
            state,
            "write:recipes",
        ))
        .merge(require_permission(
            meal_plan::read_router(),
            state,
            "read:plan",
        ))
        .merge(require_permission(
            meal_plan::write_router(),
            state,
            "write:plan",
        ))
        .merge(require_permission(
            shopping_list::read_router(),
            state,
            "read:shopping",
        ))
        .merge(require_permission(
            shopping_list::write_router(),
            state,
            "write:shopping",
        ))
        // Agent API: permisos M2M dedicados (bloque 4).
        .merge(require_permission(
            agent::read_router(),
            state,
            "agent:read",
        ))
        .merge(require_permission(
            agent::plan_router(),
            state,
            "agent:plan",
        ))
        .merge(require_permission(
            agent::shopping_router(),
            state,
            "agent:shopping",
        ))
        .merge(require_permission(
            agent::write_router(),
            state,
            "agent:write",
        ));

    public.merge(protected)
}
