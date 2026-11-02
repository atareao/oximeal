//! Validación de entrada compartida por los handlers.

use chrono::NaiveDate;

use crate::domain::date::parse_iso_date;
use crate::error::{AppError, AppResult};
use crate::models::recipe::RecipeInput;

/// Categorías de receta admitidas.
pub fn is_valid_category(value: &str) -> bool {
    matches!(value, "main" | "salad" | "side" | "dessert" | "breakfast")
}

/// Tipos de comida admitidos.
pub fn is_valid_meal_type(value: &str) -> bool {
    matches!(value, "lunch" | "dinner" | "breakfast" | "snack")
}

/// Roles de plato admitidos.
pub fn is_valid_role(value: &str) -> bool {
    matches!(value, "main" | "salad" | "side")
}

/// Parsea una fecha `YYYY-MM-DD`, devolviendo 400 si es inválida.
pub fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    parse_iso_date(value).ok_or_else(|| AppError::Validation(format!("fecha inválida: {value}")))
}

/// Valida el cuerpo de creación o actualización de receta. No persiste nada si falla.
///
/// Compartida por los handlers de recetas y por la importación desde URL, para
/// que una receta extraída de JSON-LD pase por las mismas reglas que una escrita
/// a mano.
pub fn validate_recipe(input: &RecipeInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "el nombre no puede estar vacío".into(),
        ));
    }
    if !is_valid_category(&input.category) {
        return Err(AppError::Validation(format!(
            "categoría inválida: {}",
            input.category
        )));
    }
    if input.ingredients.is_empty() {
        return Err(AppError::Validation(
            "la receta debe tener al menos un ingrediente".into(),
        ));
    }
    for ingredient in &input.ingredients {
        if ingredient.name.trim().is_empty() {
            return Err(AppError::Validation(
                "el nombre del ingrediente no puede estar vacío".into(),
            ));
        }
        if ingredient.amount < 0.0 {
            return Err(AppError::Validation(
                "el amount del ingrediente debe ser >= 0".into(),
            ));
        }
    }
    if let Some(servings) = input.servings
        && servings < 1
    {
        return Err(AppError::Validation("servings debe ser >= 1".into()));
    }
    if let Some(prep) = input.prep_time_minutes
        && prep < 0
    {
        return Err(AppError::Validation(
            "prep_time_minutes debe ser >= 0".into(),
        ));
    }

    Ok(())
}
