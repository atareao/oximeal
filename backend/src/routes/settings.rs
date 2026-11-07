//! Endpoints REST de ajustes del hogar.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, put};

use crate::error::{AppError, AppResult};
use crate::models::settings::{HouseholdSettings, SettingsInput};
use crate::repo;
use crate::state::AppState;

/// Router de ajustes de lectura (permiso `read:recipes`).
pub fn read_router() -> Router<AppState> {
    Router::new().route("/api/settings", get(get_settings))
}

/// Router de ajustes de escritura (permiso `write:recipes`).
pub fn write_router() -> Router<AppState> {
    Router::new().route("/api/settings", put(update_settings))
}

/// `GET /api/settings` — obtiene los ajustes del hogar.
///
/// Si no se ha guardado ninguna preferencia, devuelve `default_servings: 2`.
#[utoipa::path(
    get,
    path = "/api/settings",
    tag = "settings",
    responses(
        (status = 200, description = "Ajustes del hogar", body = HouseholdSettings),
    )
)]
pub async fn get_settings(State(state): State<AppState>) -> AppResult<Json<HouseholdSettings>> {
    let settings = repo::settings::get(&state.pool).await?;
    Ok(Json(settings))
}

/// `PUT /api/settings` — actualiza los ajustes del hogar.
///
/// Valida que `default_servings` esté en el rango 1..20; si no, devuelve `422`.
#[utoipa::path(
    put,
    path = "/api/settings",
    tag = "settings",
    request_body = SettingsInput,
    responses(
        (status = 200, description = "Ajustes actualizados", body = HouseholdSettings),
        (status = 422, description = "Valor fuera de rango (1..20)"),
    )
)]
pub async fn update_settings(
    State(state): State<AppState>,
    Json(input): Json<SettingsInput>,
) -> AppResult<Json<HouseholdSettings>> {
    if !(1..=20).contains(&input.default_servings) {
        return Err(AppError::UnprocessableEntity(
            "default_servings debe estar entre 1 y 20".into(),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let settings = repo::settings::update(&state.pool, input.default_servings, &now).await?;
    Ok(Json(settings))
}
