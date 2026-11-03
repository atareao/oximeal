//! DTOs y filas internas de etiquetas de recetas.
//!
//! Las etiquetas son libres y se identifican por su `slug` normalizado, que es
//! la clave canónica (`UNIQUE`). El `slug` recorta y colapsa espacios internos y
//! pasa a minúsculas Unicode **conservando los diacríticos**, de modo que la
//! identidad es insensible a mayúsculas y a espacios.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Referencia a una etiqueta asociada a una receta (sin recuento).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RecipeTag {
    /// Identificador único (UUID v4).
    pub id: String,
    /// Nombre de presentación de la etiqueta.
    pub name: String,
    /// Identidad canónica normalizada (única).
    pub slug: String,
}

/// Etiqueta del catálogo con su recuento de recetas asociadas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Tag {
    /// Identificador único (UUID v4).
    pub id: String,
    /// Nombre de presentación de la etiqueta.
    pub name: String,
    /// Identidad canónica normalizada (única).
    pub slug: String,
    /// Número de recetas asociadas (`0` si no la usa ninguna).
    pub recipe_count: i64,
}

/// Cuerpo de creación de una etiqueta.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct TagInput {
    /// Nombre de la etiqueta (no vacío; se normaliza a `slug`).
    pub name: String,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Tag {
            id: row.id,
            name: row.name,
            slug: row.slug,
            recipe_count: row.recipe_count,
        }
    }
}

/// Fila interna de una etiqueta con su recuento, decodificada con `FromRow`.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct TagRow {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub recipe_count: i64,
}

/// Devuelve el nombre de presentación: recorta y colapsa espacios internos,
/// conservando mayúsculas y diacríticos.
pub fn display_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normaliza un nombre de etiqueta a su `slug` canónico.
///
/// Recorta y colapsa espacios internos y convierte a minúsculas Unicode
/// (conservando los diacríticos). Un nombre vacío o solo de espacios produce un
/// `slug` vacío, que la capa de rutas rechaza antes de persistir.
pub fn slugify(name: &str) -> String {
    display_name(name).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_trims_and_collapses_spaces() {
        assert_eq!(slugify("  Sin   Gluten  "), "sin gluten");
    }

    #[test]
    fn slugify_lowercases_but_keeps_diacritics() {
        assert_eq!(slugify("Brócoli"), "brócoli");
    }

    #[test]
    fn slugify_ignores_case_and_inner_spacing() {
        assert_eq!(slugify("Vegano"), slugify("  VEGANO  "));
    }

    #[test]
    fn slugify_of_blank_is_empty() {
        assert_eq!(slugify("   "), "");
    }

    #[test]
    fn display_name_preserves_case() {
        assert_eq!(display_name("  Sin   Gluten  "), "Sin Gluten");
    }
}
