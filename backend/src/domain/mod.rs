//! Dominio puro de OxiMeal (sin IO).
//!
//! Contiene la lógica determinista de canonicalización de unidades,
//! normalización de nombres, categorización por pasillo y consolidación
//! de listas de la compra. Es 100 % testeable y no depende de base de datos
//! ni de red.

pub mod category;
pub mod consolidate;
pub mod date;
pub mod ingredient;
pub mod ingredient_parse;
pub mod normalize;
pub mod schema_org;
pub mod units;
