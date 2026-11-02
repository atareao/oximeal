//! DTOs de la Agent API: contrato HTTP estable que consume el asistente Valet.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::ingredient::IngredientInput;
use crate::models::recipe::RecipeSummary;
use crate::models::shopping_list::ShoppingListGroup;

/// Receta planificada recientemente, con su última fecha de planificación.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RecentRecipe {
    /// Identificador único de la receta.
    pub id: String,
    /// Nombre de la receta.
    pub name: String,
    /// Categoría (`main|salad|side|dessert|breakfast`).
    pub category: String,
    /// Última fecha en que se planificó (`YYYY-MM-DD`).
    pub last_planned: String,
}

/// Respuesta de `GET /api/agent/recent-recipes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RecentRecipesResponse {
    /// Recetas planificadas, de más reciente a más antigua.
    pub recipes: Vec<RecentRecipe>,
}

/// Respuesta de `GET /api/agent/search-recipes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SearchRecipesResponse {
    /// Coincidencias (hasta el límite configurado).
    pub recipes: Vec<RecipeSummary>,
}

/// Parámetros de `GET /api/agent/recent-recipes`.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct RecentRecipesQuery {
    /// Ventana de días hacia atrás (por defecto 7).
    #[serde(default)]
    pub days: Option<u32>,
}

/// Parámetros de `GET /api/agent/search-recipes`.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct SearchRecipesQuery {
    /// Texto de búsqueda (FTS5 si >= 3 caracteres, LIKE en caso contrario).
    #[serde(default)]
    pub query: Option<String>,
    /// Filtro opcional por categoría.
    #[serde(default)]
    pub category: Option<String>,
}

/// Entrada propuesta por el agente para un slot del plan.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StructuredPlanEntry {
    /// Fecha en formato `YYYY-MM-DD`.
    pub date: String,
    /// Tipo de comida (`lunch|dinner|breakfast|snack`).
    pub meal_type: String,
    /// Identificador de una receta existente.
    pub recipe_id: String,
    /// Rol del plato (`main|salad|side`; por defecto `main`).
    #[serde(default = "default_role")]
    pub role: String,
    /// Raciones (por defecto 2).
    #[serde(default = "default_servings")]
    pub servings: i64,
}

/// Valor por defecto de `role`.
fn default_role() -> String {
    "main".to_string()
}

/// Valor por defecto de `servings`.
fn default_servings() -> i64 {
    2
}

/// Plan semanal estructurado completo.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StructuredPlan {
    /// Entradas del plan a fijar.
    pub entries: Vec<StructuredPlanEntry>,
}

/// Respuesta de `POST /api/agent/weekly-plan`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WeeklyPlanResponse {
    /// Número de entradas persistidas.
    pub created: i64,
    /// Lista de la compra reconstruida a partir del plan.
    pub shopping_list: Vec<ShoppingListGroup>,
}

/// Cuerpo de `POST /api/agent/shopping-items`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AgentShoppingItemsInput {
    /// Ingredientes a añadir o sumar a la lista.
    pub items: Vec<IngredientInput>,
}

/// Respuesta de `DELETE /api/agent/shopping/completed`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RemovedResponse {
    /// Número de ítems eliminados.
    pub removed: u64,
}
