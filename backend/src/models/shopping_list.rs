//! DTOs y filas internas de la lista de la compra.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Ítem individual de la lista de la compra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ShoppingItem {
    /// Identificador único.
    pub id: String,
    /// Nombre de presentación.
    pub name: String,
    /// Cantidad.
    pub amount: f64,
    /// Unidad.
    pub unit: String,
    /// Categoría de pasillo.
    pub category: String,
    /// Si ya se ha comprado.
    pub checked: bool,
    /// Última actualización (texto tal cual lo produce SQLite).
    pub updated_at: String,
    /// Identificador de cliente del alta idempotente (si lo hubo).
    pub client_id: Option<String>,
}

/// Grupo de ítems por categoría de pasillo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ShoppingListGroup {
    /// Categoría de pasillo.
    pub category: String,
    /// Ítems de la categoría.
    pub items: Vec<ShoppingItem>,
}

/// Cuerpo de alta manual de un ítem de la compra.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ShoppingItemInput {
    /// Nombre del producto.
    pub name: String,
    /// Cantidad.
    pub amount: f64,
    /// Unidad.
    pub unit: String,
    /// Categoría explícita (si falta, se infiere por heurística).
    #[serde(default)]
    pub category: Option<String>,
    /// Identificador de cliente opcional para que el alta sea idempotente.
    #[serde(default)]
    pub client_id: Option<String>,
}

/// Cuerpo de actualización parcial de un ítem.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ShoppingItemPatch {
    /// Nuevo estado de comprado.
    #[serde(default)]
    pub checked: Option<bool>,
    /// Nueva cantidad.
    #[serde(default)]
    pub amount: Option<f64>,
}

/// Cuerpo (opcional) de consolidación desde el plan.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct ConsolidateInput {
    /// Inicio del rango (`YYYY-MM-DD`); si falta se usa la semana actual.
    #[serde(default)]
    pub start_date: Option<String>,
    /// Fin del rango (`YYYY-MM-DD`); si falta se usa la semana actual.
    #[serde(default)]
    pub end_date: Option<String>,
}
