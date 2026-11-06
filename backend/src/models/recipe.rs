//! DTOs y filas internas de recetas.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::ingredient::IngredientInput;
use crate::error::{AppError, AppResult};
use crate::models::tag::RecipeTag;

/// Campos de resumen de una receta (listados).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RecipeSummary {
    /// Identificador único (UUID v4).
    pub id: String,
    /// Nombre de la receta.
    pub name: String,
    /// Categoría (`main|salad|side|dessert|breakfast`).
    pub category: String,
    /// Tipo de dieta (p. ej. `mediterranean`).
    pub diet_type: String,
    /// Tiempo de preparación en minutos (opcional).
    pub prep_time_minutes: Option<i64>,
    /// Número de raciones.
    pub servings: i64,
    /// Fecha de creación (texto tal cual lo produce SQLite).
    pub created_at: String,
    /// Etiquetas asociadas a la receta.
    pub tags: Vec<RecipeTag>,
}

/// Receta completa con ingredientes e instrucciones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Recipe {
    /// Identificador único (UUID v4).
    pub id: String,
    /// Nombre de la receta.
    pub name: String,
    /// Categoría (`main|salad|side|dessert|breakfast`).
    pub category: String,
    /// Tipo de dieta.
    pub diet_type: String,
    /// Tiempo de preparación en minutos (opcional).
    pub prep_time_minutes: Option<i64>,
    /// Número de raciones.
    pub servings: i64,
    /// Fecha de creación.
    pub created_at: String,
    /// Ingredientes de la receta.
    pub ingredients: Vec<IngredientInput>,
    /// Instrucciones de elaboración (opcional).
    pub instructions: Option<String>,
    /// URL de origen (opcional).
    pub source_url: Option<String>,
    /// Etiquetas asociadas a la receta.
    pub tags: Vec<RecipeTag>,
}

/// Cuerpo de creación de receta.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecipeInput {
    /// Nombre de la receta (no vacío).
    pub name: String,
    /// Categoría válida.
    pub category: String,
    /// Tipo de dieta (por defecto `mediterranean`).
    #[serde(default)]
    pub diet_type: Option<String>,
    /// Tiempo de preparación en minutos (>= 0 si se indica).
    #[serde(default)]
    pub prep_time_minutes: Option<i64>,
    /// Número de raciones (>= 1 si se indica).
    #[serde(default)]
    pub servings: Option<i64>,
    /// Ingredientes (al menos uno).
    pub ingredients: Vec<IngredientInput>,
    /// Instrucciones de elaboración.
    #[serde(default)]
    pub instructions: Option<String>,
    /// URL de origen.
    #[serde(default)]
    pub source_url: Option<String>,
    /// Nombres de las etiquetas de la receta (se crean al vuelo por `slug` y
    /// reemplazan el conjunto anterior; por defecto, vacío).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Activa el etiquetado automático de la receta (por defecto, `true`).
    #[serde(default = "default_auto_tags")]
    pub auto_tags: bool,
    /// Nombres o `slug` de etiquetas automáticas que no deben asignarse a esta
    /// receta (por defecto, vacío).
    #[serde(default)]
    pub suppressed_auto_tags: Vec<String>,
}

/// Valor por defecto de [`RecipeInput::auto_tags`]: el etiquetado automático
/// está activo salvo que el cliente lo desactive explícitamente.
fn default_auto_tags() -> bool {
    true
}

/// Parámetros de consulta del listado de recetas.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct RecipeQuery {
    /// Texto de búsqueda (FTS5 si >= 3 caracteres, LIKE en caso contrario).
    #[serde(default)]
    pub q: Option<String>,
    /// Filtro por categoría.
    #[serde(default)]
    pub category: Option<String>,
    /// Filtro por etiqueta (`id` o `slug`).
    #[serde(default)]
    pub tag: Option<String>,
    /// Página (1-indexada).
    #[serde(default)]
    pub page: Option<u32>,
    /// Tamaño de página (máximo 100).
    #[serde(default)]
    pub page_size: Option<u32>,
}

/// Respuesta paginada del listado de recetas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RecipeListResponse {
    /// Recetas de la página.
    pub items: Vec<RecipeSummary>,
    /// Total de recetas que cumplen el filtro.
    pub total: i64,
    /// Página actual.
    pub page: u32,
    /// Tamaño de página.
    pub page_size: u32,
}

/// Fila interna de resumen (`recipes`), decodificada con `FromRow`.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct RecipeSummaryRow {
    pub id: String,
    pub name: String,
    pub category: String,
    pub diet_type: String,
    pub prep_time_minutes: Option<i64>,
    pub servings: i64,
    pub created_at: String,
}

impl RecipeSummaryRow {
    /// Convierte la fila en el DTO de resumen, adjuntando sus etiquetas.
    pub(crate) fn into_summary(self, tags: Vec<RecipeTag>) -> RecipeSummary {
        RecipeSummary {
            id: self.id,
            name: self.name,
            category: self.category,
            diet_type: self.diet_type,
            prep_time_minutes: self.prep_time_minutes,
            servings: self.servings,
            created_at: self.created_at,
            tags,
        }
    }
}

/// Fila interna completa (`recipes`), decodificada con `FromRow`.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct RecipeDetailRow {
    pub id: String,
    pub name: String,
    pub category: String,
    pub diet_type: String,
    pub prep_time_minutes: Option<i64>,
    pub servings: i64,
    pub created_at: String,
    pub ingredients: String,
    pub instructions: Option<String>,
    pub source_url: Option<String>,
}

impl RecipeDetailRow {
    /// Convierte la fila en el DTO de receta completa, adjuntando sus etiquetas.
    ///
    /// Deserializa la columna JSON de ingredientes; si el contenido almacenado
    /// está corrupto, se trata como error interno.
    pub(crate) fn into_recipe(self, tags: Vec<RecipeTag>) -> AppResult<Recipe> {
        let ingredients: Vec<IngredientInput> = serde_json::from_str(&self.ingredients)
            .map_err(|error| AppError::Internal(anyhow!("ingredientes JSON inválidos: {error}")))?;

        Ok(Recipe {
            id: self.id,
            name: self.name,
            category: self.category,
            diet_type: self.diet_type,
            prep_time_minutes: self.prep_time_minutes,
            servings: self.servings,
            created_at: self.created_at,
            ingredients,
            instructions: self.instructions,
            source_url: self.source_url,
            tags,
        })
    }
}
