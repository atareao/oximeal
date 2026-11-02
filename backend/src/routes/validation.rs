//! Validación de entrada compartida por los handlers.

use chrono::NaiveDate;

use crate::domain::date::parse_iso_date;
use crate::error::AppError;

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
