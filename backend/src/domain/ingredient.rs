//! Tipos de ingrediente de entrada y de receta.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Ingrediente tal como llega en una petición (puede traer categoría explícita).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct IngredientInput {
    /// Nombre original tal como lo escribió el usuario.
    pub name: String,
    /// Cantidad numérica.
    pub amount: f64,
    /// Unidad textual (p. ej. "kg", "ud", "al gusto").
    pub unit: String,
    /// Categoría de pasillo explícita (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Ingrediente de una receta (con categoría obligatoria).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    /// Nombre del ingrediente.
    pub name: String,
    /// Cantidad numérica.
    pub amount: f64,
    /// Unidad textual.
    pub unit: String,
    /// Categoría de pasillo.
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_sin_categoria_es_none() {
        let json = r#"{"name":"Tomate","amount":2.0,"unit":"ud"}"#;
        let input: IngredientInput = serde_json::from_str(json).expect("json válido");
        assert_eq!(input.name, "Tomate");
        assert_eq!(input.category, None);
    }

    #[test]
    fn input_roundtrip_con_categoria() {
        let input = IngredientInput {
            name: "Sal".into(),
            amount: 1.0,
            unit: "pizca".into(),
            category: Some("Despensa".into()),
        };
        let json = serde_json::to_string(&input).expect("serializa");
        assert!(json.contains("\"category\":\"Despensa\""));
        let back: IngredientInput = serde_json::from_str(&json).expect("deserializa");
        assert_eq!(input, back);
    }

    #[test]
    fn ingredient_roundtrip() {
        let ing = Ingredient {
            name: "Leche".into(),
            amount: 500.0,
            unit: "ml".into(),
            category: "Refrigerados".into(),
        };
        let json = serde_json::to_string(&ing).expect("serializa");
        let back: Ingredient = serde_json::from_str(&json).expect("deserializa");
        assert_eq!(ing, back);
    }
}
