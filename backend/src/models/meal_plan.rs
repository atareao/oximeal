//! DTOs y filas internas de la planificación de comidas.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::recipe::RecipeSummary;
use crate::models::tag::RecipeTag;

/// Vista completa de una entrada del plan, con su receta embebida.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MealPlanEntryView {
    /// Identificador único de la entrada.
    pub id: String,
    /// Fecha en formato `YYYY-MM-DD`.
    pub date: String,
    /// Tipo de comida (`lunch|dinner|breakfast|snack`).
    pub meal_type: String,
    /// Rol del plato en la comida (`main|salad|side`).
    pub role: String,
    /// Raciones asignadas.
    pub servings: i64,
    /// Receta asignada (resumen).
    pub recipe: RecipeSummary,
}

/// Cuerpo de creación de una entrada del plan.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MealPlanInput {
    /// Fecha en formato `YYYY-MM-DD`.
    pub date: String,
    /// Tipo de comida (`lunch|dinner|breakfast|snack`).
    pub meal_type: String,
    /// Identificador de una receta existente.
    pub recipe_id: String,
    /// Rol del plato (`main|salad|side`).
    pub role: String,
    /// Raciones (por defecto 2).
    #[serde(default)]
    pub servings: Option<i64>,
}

/// Cuerpo de edición de las raciones de una entrada del plan.
///
/// `servings` es opcional a nivel de deserialización para poder responder `400`
/// (y no `422`) cuando el campo falta o trae un valor inválido.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MealPlanPatchInput {
    /// Nuevas raciones; debe ser `>= 1`.
    #[serde(default)]
    pub servings: Option<i64>,
}

/// Parámetros de consulta del plan por rango de fechas.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MealPlanQuery {
    /// Fecha de inicio (`YYYY-MM-DD`).
    pub start_date: String,
    /// Fecha de fin (`YYYY-MM-DD`).
    pub end_date: String,
}

/// Fila interna del `JOIN` entre plan y recetas, decodificada con `FromRow`.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct MealPlanRow {
    pub id: String,
    pub date: String,
    pub meal_type: String,
    pub role: String,
    pub servings: i64,
    pub recipe_id: String,
    pub recipe_name: String,
    pub recipe_category: String,
    pub recipe_diet_type: String,
    pub recipe_prep_time_minutes: Option<i64>,
    pub recipe_servings: i64,
    pub recipe_created_at: String,
}

impl MealPlanRow {
    /// Convierte la fila en la vista de la entrada del plan, adjuntando las
    /// etiquetas de la receta embebida.
    pub(crate) fn into_view(self, tags: Vec<RecipeTag>) -> MealPlanEntryView {
        MealPlanEntryView {
            id: self.id,
            date: self.date,
            meal_type: self.meal_type,
            role: self.role,
            servings: self.servings,
            recipe: RecipeSummary {
                id: self.recipe_id,
                name: self.recipe_name,
                category: self.recipe_category,
                diet_type: self.recipe_diet_type,
                prep_time_minutes: self.recipe_prep_time_minutes,
                servings: self.recipe_servings,
                created_at: self.recipe_created_at,
                tags,
            },
        }
    }
}
