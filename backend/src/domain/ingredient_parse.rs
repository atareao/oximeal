//! Desglose de una línea de ingrediente en cantidad, unidad y nombre.
//!
//! Helper **puro** (sin IO, sin panic) que interpreta el texto libre de un
//! `recipeIngredient` de Schema.org: separa una cantidad inicial (entero,
//! decimal con punto o coma, fracción `a/b` o mixta `a b/c`), reconoce unidades
//! conocidas de una lista blanca normalizada y conserva el resto como nombre.

use super::ingredient::IngredientInput;
use super::units::normalize_unit;

/// Cantidad y unidad por defecto cuando no se reconoce ninguna cantidad.
const DEFAULT_AMOUNT: f64 = 1.0;
/// Unidad por defecto (recuento de unidades).
const DEFAULT_UNIT: &str = "ud";
/// Conector que se elimina entre la unidad y el nombre («de»).
const CONNECTOR: &str = "de";

/// Lista blanca de unidades conocidas: `(forma normalizada, forma canónica)`.
///
/// La forma canónica es el singular que se persiste; la clave se compara tras
/// [`normalize_unit`] (minúsculas, sin diacríticos, espacios colapsados).
///
/// # Follow-up: alinear con [`super::units::canonicalize`]
///
/// Se evaluó reutilizar la tabla de alias de `units::canonicalize` (que ya
/// reconoce `gramos`, `litro(s)`, `u`/`unidad(es)`, `cda`, `cdta`, ...) para
/// reconocer sinónimos comunes. Se descarta por ahora: `canonicalize` mezcla
/// familia física + conversión de magnitud y usa su propia unidad de
/// presentación (`cda`, `ml`, `pizca`), incompatible con el singular que aquí
/// se persiste sin convertir (`kg` se conserva como `kg`). Unificarlo exigiría
/// refactorizar `units.rs` a una tabla única `(alias, familia, factor,
/// presentación)` compartida; queda como follow-up para no arriesgar la
/// canonicalización (17 tests) en este cambio.
const KNOWN_UNITS: &[(&str, &str)] = &[
    // Masa
    ("g", "g"),
    // `gr` es el alias idiomático de gramos en español; necesario además para
    // reconocer variantes en mayúsculas («40 GR» → «g»).
    ("gr", "g"),
    ("gramo", "g"),
    ("gramos", "g"),
    ("kg", "kg"),
    ("kilo", "kg"),
    ("kilos", "kg"),
    ("kilogramo", "kg"),
    ("kilogramos", "kg"),
    // Volumen
    ("ml", "ml"),
    ("mililitro", "ml"),
    ("mililitros", "ml"),
    ("l", "l"),
    ("lt", "l"),
    ("litro", "l"),
    ("litros", "l"),
    ("cl", "cl"),
    // Recuento
    ("ud", "ud"),
    ("uds", "ud"),
    ("u", "ud"),
    ("unidad", "ud"),
    ("unidades", "ud"),
    ("docena", "docena"),
    ("docenas", "docena"),
    // Cucharas
    ("cda", "cucharada"),
    ("cucharada", "cucharada"),
    ("cucharadas", "cucharada"),
    ("cdta", "cucharadita"),
    ("cucharadita", "cucharadita"),
    ("cucharaditas", "cucharadita"),
    // Otras
    ("taza", "taza"),
    ("tazas", "taza"),
    ("pizca", "pizca"),
    ("pizcas", "pizca"),
];

/// Desglosa una línea de ingrediente en sus componentes.
///
/// Sin cantidad detectable devuelve `amount = 1.0`, `unit = "ud"` y la línea
/// completa recortada como `name`; si tras reconocer una unidad el nombre queda
/// vacío, se aplica el mismo comportamiento de reserva (nunca se pierde texto).
pub fn parse_ingredient(line: &str) -> IngredientInput {
    let collapsed = collapse_whitespace(line);
    if collapsed.is_empty() {
        return default_ingredient("");
    }

    let tokens: Vec<&str> = collapsed.split(' ').collect();
    let Some((amount, consumed)) = parse_amount(&tokens) else {
        return default_ingredient(&collapsed);
    };

    let rest = tokens.get(consumed..).unwrap_or_default();
    build_with_amount(amount, rest).unwrap_or_else(|| default_ingredient(&collapsed))
}

/// Construye un ingrediente con `amount` conocido a partir del resto de tokens.
///
/// Devuelve `None` si el nombre resultante queda vacío (el llamador aplica la
/// reserva con la línea completa).
fn build_with_amount(amount: f64, rest: &[&str]) -> Option<IngredientInput> {
    let (unit, name_tokens) = match rest.first() {
        Some(token) => match canonical_unit(token) {
            Some(unit) => {
                let name_start = if rest.get(1).is_some_and(|next| is_connector(next)) {
                    2
                } else {
                    1
                };
                (unit, rest.get(name_start..).unwrap_or_default())
            }
            None => (DEFAULT_UNIT.to_string(), rest),
        },
        None => (DEFAULT_UNIT.to_string(), rest),
    };

    let name = name_tokens.join(" ");
    if name.is_empty() {
        return None;
    }
    Some(IngredientInput {
        name,
        amount,
        unit,
        category: None,
    })
}

/// Ingrediente de reserva: línea completa como nombre, `amount = 1`, `unit = "ud"`.
fn default_ingredient(name: &str) -> IngredientInput {
    IngredientInput {
        name: name.to_string(),
        amount: DEFAULT_AMOUNT,
        unit: DEFAULT_UNIT.to_string(),
        category: None,
    }
}

/// Colapsa cualquier secuencia de espacios en blanco en un único espacio.
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Indica si un token es el conector «de» (insensible a mayúsculas/diacríticos).
fn is_connector(token: &str) -> bool {
    normalize_unit(token) == CONNECTOR
}

/// Devuelve la unidad canónica (singular) de un token, si está en la lista blanca.
fn canonical_unit(token: &str) -> Option<String> {
    let normalized = normalize_unit(token);
    KNOWN_UNITS
        .iter()
        .find(|(alias, _)| *alias == normalized)
        .map(|(_, canonical)| (*canonical).to_string())
}

/// Interpreta la cantidad inicial de la lista de tokens.
///
/// Devuelve la cantidad y cuántos tokens consume (1 para entero/decimal/fracción,
/// 2 para la forma mixta `a b/c`). `None` si el primer token no es una cantidad.
fn parse_amount(tokens: &[&str]) -> Option<(f64, usize)> {
    let first = *tokens.first()?;

    if let Some(fraction) = parse_fraction(first) {
        return Some((fraction, 1));
    }

    let value = parse_number(first)?;
    if value.fract() == 0.0
        && let Some(second) = tokens.get(1)
        && let Some(fraction) = parse_fraction(second)
    {
        return Some((value + fraction, 2));
    }
    Some((value, 1))
}

/// Interpreta un número entero o decimal (punto o coma como separador).
fn parse_number(token: &str) -> Option<f64> {
    if token.is_empty()
        || !token
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return None;
    }
    token.replace(',', ".").parse::<f64>().ok()
}

/// Interpreta una fracción `a/b` (denominador distinto de cero).
fn parse_fraction(token: &str) -> Option<f64> {
    let (numerator, denominator) = token.split_once('/')?;
    let numerator = parse_number(numerator)?;
    let denominator = parse_number(denominator)?;
    if denominator == 0.0 {
        return None;
    }
    Some(numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Comprueba nombre, cantidad y unidad de una línea de ingrediente.
    fn assert_parse(line: &str, name: &str, amount: f64, unit: &str) {
        let parsed = parse_ingredient(line);
        assert_eq!(parsed.name, name, "nombre de {line:?}");
        assert!(
            (parsed.amount - amount).abs() < 1e-9,
            "cantidad de {line:?}: {} != {amount}",
            parsed.amount
        );
        assert_eq!(parsed.unit, unit, "unidad de {line:?}");
        assert_eq!(parsed.category, None, "categoría de {line:?}");
    }

    #[test]
    fn cantidad_con_unidad() {
        assert_parse("40 g de queso parmesano", "queso parmesano", 40.0, "g");
    }

    #[test]
    fn cantidad_sin_unidad() {
        assert_parse("1 calabacín", "calabacín", 1.0, "ud");
    }

    #[test]
    fn fraccion() {
        assert_parse("1/2 taza de harina", "harina", 0.5, "taza");
    }

    #[test]
    fn sin_cantidad_detectable() {
        assert_parse("sal al gusto", "sal al gusto", 1.0, "ud");
    }

    #[test]
    fn decimal_con_coma() {
        assert_parse("1,5 kg de harina", "harina", 1.5, "kg");
    }

    #[test]
    fn cantidad_mixta_y_plural_normalizado() {
        assert_parse("1 1/2 tazas de azúcar", "azúcar", 1.5, "taza");
    }

    #[test]
    fn cantidad_sin_unidad_conocida_conserva_el_resto() {
        assert_parse("2 dientes de ajo", "dientes de ajo", 2.0, "ud");
    }

    #[test]
    fn linea_vacia_no_produce_panico() {
        let parsed = parse_ingredient("   ");
        assert_eq!(parsed.name, "");
        assert_eq!(parsed.unit, "ud");
        assert!((parsed.amount - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unidad_conocida_sin_conector() {
        assert_parse("40 g queso parmesano", "queso parmesano", 40.0, "g");
    }

    #[test]
    fn unidad_con_mayusculas_se_normaliza() {
        assert_parse("40 GR de queso", "queso", 40.0, "g");
    }

    #[test]
    fn cantidad_y_unidad_sin_nombre_caen_al_fallback() {
        // Documenta el comportamiento fijado: si tras la unidad no queda nombre,
        // se conserva la línea íntegra con `amount = 1` y `unit = "ud"`.
        assert_parse("1 l", "1 l", 1.0, "ud");
        assert_parse("40 g", "40 g", 1.0, "ud");
    }

    #[test]
    fn conector_de_con_plural() {
        assert_parse("2 tazas de arroz", "arroz", 2.0, "taza");
    }

    #[test]
    fn sinonimo_masa_forma_larga() {
        assert_parse(
            "150 gramos de pechuga de pollo",
            "pechuga de pollo",
            150.0,
            "g",
        );
    }

    #[test]
    fn sinonimo_masa_brocoli() {
        assert_parse("60 gramos de brócoli", "brócoli", 60.0, "g");
    }

    #[test]
    fn cantidad_sin_unidad_zanahoria() {
        assert_parse("1 zanahoria", "zanahoria", 1.0, "ud");
    }

    #[test]
    fn sinonimo_masa_queso_crema() {
        assert_parse("50 gramos de queso crema", "queso crema", 50.0, "g");
    }

    #[test]
    fn sinonimo_masa_copos_de_maiz() {
        assert_parse("50 gramos de copos de maíz", "copos de maíz", 50.0, "g");
    }

    #[test]
    fn sinonimo_masa_kilo() {
        assert_parse("1 kilo de patatas", "patatas", 1.0, "kg");
    }

    #[test]
    fn sinonimo_volumen_litros() {
        assert_parse("2 litros de leche", "leche", 2.0, "l");
    }

    #[test]
    fn sinonimo_volumen_mililitros() {
        assert_parse("500 mililitros de agua", "agua", 500.0, "ml");
    }

    #[test]
    fn sinonimo_recuento_unidades() {
        assert_parse("3 unidades de huevo", "huevo", 3.0, "ud");
    }

    #[test]
    fn sinonimo_recuento_docenas() {
        assert_parse("2 docenas de huevos", "huevos", 2.0, "docena");
    }

    #[test]
    fn sinonimo_cucharada_cda() {
        assert_parse("2 cda de aceite", "aceite", 2.0, "cucharada");
    }

    #[test]
    fn sinonimo_cucharadita_cdta() {
        assert_parse("1 cdta de sal", "sal", 1.0, "cucharadita");
    }
}
