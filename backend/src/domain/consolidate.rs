//! Consolidación determinista de ingredientes en líneas de la compra.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::category::{CATEGORY_ORDER, categorize};
use super::ingredient::IngredientInput;
use super::normalize::normalize_name;
use super::units::{Canonical, UnitFamily, canonicalize};

/// Línea borrador de la lista de la compra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoppingListItemDraft {
    /// Nombre de presentación (primera aparición del grupo).
    pub name: String,
    /// Cantidad consolidada.
    pub amount: f64,
    /// Unidad de presentación canónica.
    pub unit: String,
    /// Categoría de pasillo.
    pub category: String,
}

/// Redondea a dos decimales para evitar ruido de coma flotante.
pub(crate) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Posición de una categoría en el orden de pasillos de la compra.
///
/// Deriva de [`CATEGORY_ORDER`]; las categorías desconocidas se sitúan al final
/// (equivalente a `"Otros"`).
fn category_rank(category: &str) -> usize {
    CATEGORY_ORDER
        .iter()
        .position(|&known| known == category)
        .unwrap_or(CATEGORY_ORDER.len() - 1)
}

/// Subgrupo de un ingrediente por familia/unidad canónica.
struct SubGroup {
    family: UnitFamily,
    display_unit: String,
    total: f64,
    count: usize,
    significant: bool,
}

impl SubGroup {
    /// Cantidad final de la línea.
    fn amount(&self) -> f64 {
        match self.family {
            UnitFamily::ToTaste => 0.0,
            // Unidad desconocida: si no hay cantidades significativas, se cuenta
            // el número de ocurrencias.
            UnitFamily::Unknown => {
                if self.significant {
                    round2(self.total)
                } else {
                    self.count as f64
                }
            }
            _ => round2(self.total),
        }
    }
}

/// Acumulador de un ingrediente (mismo nombre normalizado).
struct Group {
    name: String,
    category: Option<String>,
    subs: Vec<SubGroup>,
    sub_index: HashMap<(UnitFamily, String), usize>,
}

impl Group {
    fn add(&mut self, canonical: Canonical) {
        let key = (canonical.family, canonical.display_unit.clone());
        let index = match self.sub_index.get(&key) {
            Some(&index) => index,
            None => {
                self.subs.push(SubGroup {
                    family: canonical.family,
                    display_unit: canonical.display_unit,
                    total: 0.0,
                    count: 0,
                    significant: false,
                });
                let index = self.subs.len() - 1;
                self.sub_index.insert(key, index);
                index
            }
        };
        let sub = &mut self.subs[index];
        sub.total += canonical.amount;
        sub.count += 1;
        if canonical.amount != 0.0 {
            sub.significant = true;
        }
    }
}

/// Consolida una lista de ingredientes en líneas de la compra.
///
/// Agrupa por nombre normalizado, subagrupa por familia/unidad canónica, suma
/// cantidades (redondeando a 2 decimales) y respeta las familias no aditivas
/// (`ToTaste` y `Unknown`). El resultado se ordena de forma estable por pasillo
/// y, después, por nombre.
pub fn consolidate(inputs: &[IngredientInput]) -> Vec<ShoppingListItemDraft> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Group> = HashMap::new();

    for input in inputs {
        let key = normalize_name(&input.name);
        let canonical = canonicalize(input.amount, &input.unit);
        let group = groups.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            Group {
                name: input.name.clone(),
                category: input.category.clone(),
                subs: Vec::new(),
                sub_index: HashMap::new(),
            }
        });
        group.add(canonical);
    }

    let mut drafts: Vec<ShoppingListItemDraft> = Vec::new();
    for key in &order {
        let Some(group) = groups.get(key) else {
            continue;
        };
        let category = group
            .category
            .clone()
            .unwrap_or_else(|| categorize(key).to_string());
        for sub in &group.subs {
            let unit = if sub.family == UnitFamily::ToTaste {
                "al gusto".to_string()
            } else {
                sub.display_unit.clone()
            };
            drafts.push(ShoppingListItemDraft {
                name: group.name.clone(),
                amount: sub.amount(),
                unit,
                category: category.clone(),
            });
        }
    }

    drafts.sort_by(|a, b| {
        category_rank(&a.category)
            .cmp(&category_rank(&b.category))
            .then_with(|| a.name.cmp(&b.name))
    });

    drafts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(name: &str, amount: f64, unit: &str) -> IngredientInput {
        IngredientInput {
            name: name.into(),
            amount,
            unit: unit.into(),
            category: None,
        }
    }

    #[test]
    fn suma_tomate_misma_unidad() {
        let out = consolidate(&[input("Tomate", 2.0, "ud"), input("tomates", 3.0, "ud")]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Tomate");
        assert!((out[0].amount - 5.0).abs() < 1e-9);
        assert_eq!(out[0].unit, "ud");
        assert_eq!(out[0].category, "Verdulería");
    }

    #[test]
    fn familias_incompatibles_separadas() {
        let out = consolidate(&[input("leche", 500.0, "ml"), input("leche", 1.0, "ud")]);
        assert_eq!(out.len(), 2);
        let units: Vec<&str> = out.iter().map(|d| d.unit.as_str()).collect();
        assert!(units.contains(&"ml"));
        assert!(units.contains(&"ud"));
    }

    #[test]
    fn no_aditivos_al_gusto_y_pizca() {
        let out = consolidate(&[input("sal", 0.0, "al gusto"), input("sal", 1.0, "pizca")]);
        assert_eq!(out.len(), 2);
        let to_taste = out.iter().find(|d| d.unit == "al gusto").expect("al gusto");
        assert!((to_taste.amount - 0.0).abs() < 1e-9);
        let pinch = out.iter().find(|d| d.unit == "pizca").expect("pizca");
        assert!((pinch.amount - 1.0).abs() < 1e-9);
        assert_eq!(to_taste.category, "Despensa");
    }

    #[test]
    fn misma_familia_distintas_unidades() {
        let out = consolidate(&[input("harina", 500.0, "g"), input("harina", 1.0, "kg")]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].unit, "g");
        assert!((out[0].amount - 1500.0).abs() < 1e-9);
        assert_eq!(out[0].category, "Despensa");
    }

    #[test]
    fn redondeo_a_dos_decimales() {
        let out = consolidate(&[input("aceite", 0.1, "l"), input("aceite", 0.2, "l")]);
        assert_eq!(out.len(), 1);
        assert!((out[0].amount - 300.0).abs() < 1e-9);
    }

    #[test]
    fn categoria_explicita_tiene_prioridad() {
        let mut a = input("queso", 1.0, "ud");
        a.category = Some("Despensa".into());
        let out = consolidate(&[a]);
        assert_eq!(out[0].category, "Despensa");
    }

    #[test]
    fn orden_estable_por_categoria_y_nombre() {
        let out = consolidate(&[
            input("arroz", 1.0, "kg"),
            input("tomate", 1.0, "ud"),
            input("merluza", 1.0, "kg"),
        ]);
        let cats: Vec<&str> = out.iter().map(|d| d.category.as_str()).collect();
        assert_eq!(cats, vec!["Verdulería", "Pescadería", "Despensa"]);
    }

    #[test]
    fn unidad_desconocida_sin_cantidades_cuenta_ocurrencias() {
        let out = consolidate(&[input("cajas", 0.0, "caja"), input("cajas", 0.0, "caja")]);
        assert_eq!(out.len(), 1);
        assert!((out[0].amount - 2.0).abs() < 1e-9);
        assert_eq!(out[0].unit, "caja");
    }
}
