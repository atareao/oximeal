//! DTOs y filas internas de ajustes del hogar.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Ajustes del hogar (singleton).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HouseholdSettings {
    /// Número de comensales habituales (1..20).
    pub default_servings: i64,
    /// Fecha de última actualización (ISO-8601 UTC).
    /// Será `null` cuando no existe fila en BD y se devuelve el valor por defecto.
    pub updated_at: Option<String>,
}

/// Entrada para actualizar los ajustes del hogar.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SettingsInput {
    /// Número de comensales habituales (1..20).
    pub default_servings: i64,
}
