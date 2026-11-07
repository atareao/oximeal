//! Repositorio de ajustes del hogar (SQLite vía `sqlx`).

use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::settings::HouseholdSettings;

/// Obtiene los ajustes del hogar.
///
/// Si la fila singleton no existe, devuelve el valor por defecto (2) sin insertarlo.
pub async fn get(pool: &SqlitePool) -> AppResult<HouseholdSettings> {
    #[derive(sqlx::FromRow)]
    struct SettingsRow {
        default_servings: i64,
        updated_at: String,
    }

    let row = sqlx::query_as::<_, SettingsRow>(
        "SELECT default_servings, updated_at FROM household_settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => Ok(HouseholdSettings {
            default_servings: row.default_servings,
            updated_at: Some(row.updated_at),
        }),
        None => Ok(HouseholdSettings {
            default_servings: 2,
            updated_at: None,
        }),
    }
}

/// Actualiza los ajustes del hogar.
///
/// Inserta o actualiza la fila singleton (id = 1).
pub async fn update(pool: &SqlitePool, value: i64, now: &str) -> AppResult<HouseholdSettings> {
    sqlx::query(
        "INSERT INTO household_settings (id, default_servings, updated_at)
         VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET default_servings = ?, updated_at = ?",
    )
    .bind(value)
    .bind(now)
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(HouseholdSettings {
        default_servings: value,
        updated_at: Some(now.to_string()),
    })
}
