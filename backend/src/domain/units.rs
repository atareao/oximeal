//! Canonicalización de unidades de medida.

use super::normalize::deaccent;

/// Familia física a la que pertenece una unidad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitFamily {
    /// Masa (canónico: gramos).
    Mass,
    /// Volumen (canónico: mililitros).
    Volume,
    /// Recuento (canónico: unidades).
    Count,
    /// Cucharas (canónico: cucharadas).
    Spoon,
    /// Pizcas, puñados y manojos.
    Pinch,
    /// "Al gusto" / sin cantidad.
    ToTaste,
    /// Unidad desconocida: se conserva tal cual.
    Unknown,
}

/// Resultado de canonicalizar una cantidad y su unidad.
#[derive(Debug, Clone, PartialEq)]
pub struct Canonical {
    /// Familia física detectada.
    pub family: UnitFamily,
    /// Cantidad expresada en la unidad canónica de la familia.
    pub amount: f64,
    /// Unidad de presentación canónica.
    pub display_unit: String,
}

fn canonical(family: UnitFamily, amount: f64, display_unit: &str) -> Canonical {
    Canonical {
        family,
        amount,
        display_unit: display_unit.to_string(),
    }
}

/// Normaliza una unidad textual: minúsculas, sin diacríticos y con espacios colapsados.
///
/// Se usa tanto para canonicalizar como para construir claves de conciliación
/// (p. ej. `"Cajas"` y `" cajas "` deben coincidir).
pub(crate) fn normalize_unit(unit: &str) -> String {
    deaccent(&unit.to_lowercase())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convierte una cantidad y su unidad textual a su forma canónica.
///
/// La unidad de presentación (`display_unit`) se devuelve como `String` para poder
/// conservar tal cual la unidad original cuando esta es desconocida.
pub fn canonicalize(amount: f64, unit: &str) -> Canonical {
    let normalized = normalize_unit(unit);
    match normalized.as_str() {
        // Masa → gramos
        "g" | "gr" | "gramo" | "gramos" => canonical(UnitFamily::Mass, amount, "g"),
        "kg" | "kilo" | "kilos" | "kilogramo" | "kilogramos" => {
            canonical(UnitFamily::Mass, amount * 1000.0, "g")
        }
        "mg" => canonical(UnitFamily::Mass, amount * 0.001, "g"),

        // Volumen → mililitros
        "ml" | "mililitro" | "mililitros" => canonical(UnitFamily::Volume, amount, "ml"),
        "l" | "lt" | "litro" | "litros" => canonical(UnitFamily::Volume, amount * 1000.0, "ml"),
        "cl" => canonical(UnitFamily::Volume, amount * 10.0, "ml"),
        "dl" => canonical(UnitFamily::Volume, amount * 100.0, "ml"),

        // Recuento → unidades
        "ud" | "u" | "unidad" | "unidades" => canonical(UnitFamily::Count, amount, "ud"),
        "docena" | "docenas" => canonical(UnitFamily::Count, amount * 12.0, "ud"),

        // Cucharas → cucharadas
        "cda" | "cucharada" | "cucharadas" => canonical(UnitFamily::Spoon, amount, "cda"),
        "cdta" | "cucharadita" | "cucharaditas" => {
            canonical(UnitFamily::Spoon, amount / 3.0, "cda")
        }

        // Pizcas
        "pizca" | "pizcas" | "punado" | "punados" | "manojo" | "manojos" => {
            canonical(UnitFamily::Pinch, amount, "pizca")
        }

        // Al gusto
        "al gusto" | "a gusto" | "c/n" | "c.s." | "" => {
            canonical(UnitFamily::ToTaste, 0.0, "al gusto")
        }

        // Desconocida: se conserva la unidad original recortada
        _ => canonical(UnitFamily::Unknown, amount, unit.trim()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn gramos_sin_conversion() {
        let c = canonicalize(1500.0, "g");
        assert_eq!(c.family, UnitFamily::Mass);
        assert!(approx(c.amount, 1500.0));
        assert_eq!(c.display_unit, "g");
    }

    #[test]
    fn kilogramos_a_gramos() {
        let c = canonicalize(1.5, "kg");
        assert_eq!(c.family, UnitFamily::Mass);
        assert!(approx(c.amount, 1500.0));
        assert_eq!(c.display_unit, "g");
    }

    #[test]
    fn miligramos_a_gramos() {
        let c = canonicalize(1000.0, "mg");
        assert_eq!(c.family, UnitFamily::Mass);
        assert!(approx(c.amount, 1.0));
        assert_eq!(c.display_unit, "g");
    }

    #[test]
    fn gramo_palabra_completa() {
        let c = canonicalize(250.0, "gramos");
        assert_eq!(c.family, UnitFamily::Mass);
        assert!(approx(c.amount, 250.0));
    }

    #[test]
    fn litros_a_mililitros() {
        let c = canonicalize(2.0, "l");
        assert_eq!(c.family, UnitFamily::Volume);
        assert!(approx(c.amount, 2000.0));
        assert_eq!(c.display_unit, "ml");
    }

    #[test]
    fn centilitros_y_decilitros() {
        assert!(approx(canonicalize(1.0, "cl").amount, 10.0));
        assert!(approx(canonicalize(1.0, "dl").amount, 100.0));
        assert_eq!(canonicalize(1.0, "cl").family, UnitFamily::Volume);
    }

    #[test]
    fn docena_a_unidades() {
        let c = canonicalize(2.0, "docena");
        assert_eq!(c.family, UnitFamily::Count);
        assert!(approx(c.amount, 24.0));
        assert_eq!(c.display_unit, "ud");
    }

    #[test]
    fn cucharadita_a_cucharada() {
        let c = canonicalize(3.0, "cdta");
        assert_eq!(c.family, UnitFamily::Spoon);
        assert!(approx(c.amount, 1.0));
        assert_eq!(c.display_unit, "cda");
    }

    #[test]
    fn punado_es_pizca() {
        let c = canonicalize(1.0, "puñado");
        assert_eq!(c.family, UnitFamily::Pinch);
        assert!(approx(c.amount, 1.0));
        assert_eq!(c.display_unit, "pizca");
    }

    #[test]
    fn al_gusto_y_vacio_son_to_taste() {
        let c = canonicalize(0.0, "al gusto");
        assert_eq!(c.family, UnitFamily::ToTaste);
        assert!(approx(c.amount, 0.0));
        assert_eq!(c.display_unit, "al gusto");

        let empty = canonicalize(0.0, "");
        assert_eq!(empty.family, UnitFamily::ToTaste);
        assert_eq!(empty.display_unit, "al gusto");
    }

    #[test]
    fn insensible_a_mayusculas_acentos_y_espacios() {
        assert_eq!(canonicalize(1.0, "GRAMOS").family, UnitFamily::Mass);
        assert_eq!(canonicalize(1.0, "  Kg  ").family, UnitFamily::Mass);
        assert_eq!(canonicalize(1.0, "PIZCAS").family, UnitFamily::Pinch);
    }

    #[test]
    fn unidad_desconocida_se_conserva() {
        let c = canonicalize(3.0, "cajas");
        assert_eq!(c.family, UnitFamily::Unknown);
        assert!(approx(c.amount, 3.0));
        assert_eq!(c.display_unit, "cajas");
    }
}
