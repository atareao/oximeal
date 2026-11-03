//! Parser de recetas embebidas en HTML como JSON-LD de Schema.org.
//!
//! Módulo de **dominio puro** (sin IO): recibe el HTML descargado y extrae el
//! objeto `@type: Recipe` de los bloques `<script type="application/ld+json">`,
//! mapeándolo a [`RecipeInput`].
//!
//! # Contrato de mapeo
//!
//! - `name` → `name`.
//! - `category` se fija a `"main"` (Schema.org usa `recipeCategory`, texto
//!   libre, que no se corresponde con nuestro enum) y `diet_type` queda `None`.
//! - `recipeIngredient` (lista de textos) → `ingredients`: cada texto se toma
//!   íntegro como `name`, con `amount = 1.0` y `unit = "ud"` (el parser no
//!   interpreta cantidades).
//! - `recipeYield` → `servings` (número o cadena con entero inicial); por
//!   defecto [`DEFAULT_SERVINGS`].
//! - `prepTime` → `prep_time_minutes`; si falta, se usa `totalTime`.
//! - `recipeInstructions` (`HowToStep`/`HowToSection`/texto plano) → `instructions`
//!   aplanadas a texto, en el orden dado, unidas por `\n`.
//! - `url` → `source_url`.

use serde_json::Value;

use crate::domain::ingredient::IngredientInput;
use crate::models::recipe::RecipeInput;

/// Raciones por defecto cuando el JSON-LD no trae `recipeYield`.
pub const DEFAULT_SERVINGS: i64 = 2;

/// Categoría fija de las recetas importadas.
///
/// Schema.org usa `recipeCategory` como texto libre, que no se corresponde con
/// el enum de categorías de OxiMeal; por eso se fija a `"main"`.
const DEFAULT_CATEGORY: &str = "main";

/// Extrae la primera receta (`@type: Recipe`) de los bloques JSON-LD del HTML.
///
/// Recorre todas las etiquetas `<script type="application/ld+json">` y busca un
/// objeto `Recipe` tanto en la raíz como dentro de `@graph`. Devuelve `None`
/// cuando el HTML no contiene ninguna receta reconocible (o el JSON-LD es
/// inválido).
pub fn extract_recipe(html: &str) -> Option<RecipeInput> {
    for block in json_ld_blocks(html) {
        let Ok(value) = serde_json::from_str::<Value>(block) else {
            continue;
        };
        if let Some(recipe) = find_recipe(&value) {
            return Some(map_recipe(recipe));
        }
    }
    None
}

/// Devuelve el contenido interno de cada bloque `<script type="application/ld+json">`.
///
/// Escáner manual (sin parser HTML externo): localiza las etiquetas `<script`,
/// comprueba el atributo `type` de la etiqueta de apertura (tolerando comillas
/// simples/dobles, espacios y otros atributos) y extrae hasta `</script>`.
fn json_ld_blocks(html: &str) -> Vec<&str> {
    // Copia ASCII en minúsculas, de igual longitud en bytes, para localizar las
    // etiquetas sin alterar los índices sobre el HTML original.
    let lower = html.to_ascii_lowercase();
    let mut blocks = Vec::new();
    let mut cursor = 0usize;

    while let Some(rel) = lower[cursor..].find("<script") {
        let tag_start = cursor + rel;
        let Some(gt) = html[tag_start..].find('>') else {
            break;
        };
        let open_end = tag_start + gt + 1;
        let attributes = &html[tag_start + "<script".len()..tag_start + gt];

        if is_ld_json_script(attributes) {
            let Some(close_rel) = lower[open_end..].find("</script") else {
                break;
            };
            blocks.push(&html[open_end..open_end + close_rel]);
            cursor = open_end + close_rel + "</script".len();
        } else {
            cursor = open_end;
        }
    }

    blocks
}

/// Indica si los atributos de una etiqueta declaran `type="application/ld+json"`.
fn is_ld_json_script(attributes: &str) -> bool {
    for (name, value) in AttributesIter(attributes) {
        if name.eq_ignore_ascii_case("type")
            && value.trim().eq_ignore_ascii_case("application/ld+json")
        {
            return true;
        }
    }
    false
}

/// Iterador de atributos HTML sobre el fragmento de una etiqueta:
/// `clave`, `clave=valor`, `clave="valor"` o `clave='valor'`.
struct AttributesIter<'a>(&'a str);

impl<'a> Iterator for AttributesIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let text = self.0;
        let bytes = text.as_bytes();

        // Salta espacios previos.
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            self.0 = "";
            return None;
        }

        // Nombre del atributo.
        let name_start = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric()
                || bytes[i] == b'-'
                || bytes[i] == b'_'
                || bytes[i] == b':')
        {
            i += 1;
        }
        // Carácter desconocido: avanza un carácter completo (no un byte, para no
        // partir un carácter UTF-8 multibyte y provocar un pánico).
        if i == name_start {
            let step = text[name_start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
            self.0 = &text[name_start + step..];
            return Some(("", ""));
        }
        let name = &text[name_start..i];

        // Espacios antes de `=`.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        let mut value = "";
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let value_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                value = &text[value_start..i];
                if i < bytes.len() {
                    i += 1; // comilla de cierre
                }
            } else {
                let value_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                value = &text[value_start..i];
            }
        }

        self.0 = &text[i..];
        Some((name, value))
    }
}

/// Busca el primer objeto `Recipe` en un valor JSON, entrando en `@graph`.
fn find_recipe(value: &Value) -> Option<&Value> {
    match value {
        Value::Object(map) => {
            if has_recipe_type(map.get("@type")) {
                return Some(value);
            }
            // `@graph` puede ser un array de objetos (o un único objeto).
            if let Some(graph) = map.get("@graph")
                && let Some(found) = find_recipe(graph)
            {
                return Some(found);
            }
            None
        }
        Value::Array(items) => items.iter().find_map(find_recipe),
        _ => None,
    }
}

/// Indica si un valor `@type` es (o contiene) `"Recipe"`.
fn has_recipe_type(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(kind)) => kind.eq_ignore_ascii_case("Recipe"),
        Some(Value::Array(kinds)) => kinds.iter().any(|kind| {
            kind.as_str()
                .is_some_and(|text| text.eq_ignore_ascii_case("Recipe"))
        }),
        _ => false,
    }
}

/// Mapea un objeto JSON-LD `Recipe` a [`RecipeInput`].
///
/// Simplificación documentada: cada entrada de `recipeIngredient` se toma
/// **íntegra** como nombre del ingrediente, con `amount = 1.0` y `unit = "ud"`;
/// el parser no interpreta cantidades ni unidades.
fn map_recipe(recipe: &Value) -> RecipeInput {
    let name = recipe
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let ingredients = recipe
        .get("recipeIngredient")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(|text| IngredientInput {
                    name: text.to_string(),
                    amount: 1.0,
                    unit: "ud".to_string(),
                    category: None,
                })
                .collect()
        })
        .unwrap_or_default();

    let servings = recipe
        .get("recipeYield")
        .and_then(parse_yield)
        .unwrap_or(DEFAULT_SERVINGS);

    let prep_time_minutes = recipe
        .get("prepTime")
        .and_then(Value::as_str)
        .and_then(duration_to_minutes)
        .or_else(|| {
            recipe
                .get("totalTime")
                .and_then(Value::as_str)
                .and_then(duration_to_minutes)
        });

    let instructions = recipe
        .get("recipeInstructions")
        .and_then(flatten_instructions);

    let source_url = recipe
        .get("url")
        .and_then(Value::as_str)
        .map(str::to_string);

    RecipeInput {
        name,
        category: DEFAULT_CATEGORY.to_string(),
        diet_type: None,
        prep_time_minutes,
        servings: Some(servings),
        ingredients,
        instructions,
        source_url,
        tags: Vec::new(),
    }
}

/// Extrae las raciones de `recipeYield` (número, cadena con entero inicial o
/// array del que se toma el primer valor interpretable).
fn parse_yield(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64().filter(|servings| *servings >= 1),
        Value::String(text) => parse_leading_integer(text),
        Value::Array(items) => items.iter().find_map(parse_yield),
        _ => None,
    }
}

/// Extrae el entero inicial de un texto (`"4"`, `"4 raciones"`, `"4 servings"`).
fn parse_leading_integer(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Convierte una duración ISO-8601 (`PT#H#M`, `PT#M`, `PT#H`) a minutos.
fn duration_to_minutes(value: &str) -> Option<i64> {
    let rest = value.trim().strip_prefix("PT")?;
    let mut hours = 0i64;
    let mut minutes = 0i64;
    let mut number = String::new();
    let mut saw_unit = false;

    for character in rest.chars() {
        match character {
            '0'..='9' => number.push(character),
            'H' | 'h' => {
                hours = number.parse().ok()?;
                number.clear();
                saw_unit = true;
            }
            'M' | 'm' => {
                minutes = number.parse().ok()?;
                number.clear();
                saw_unit = true;
            }
            _ => return None,
        }
    }

    // Aritmética comprobada: `PT<i64::MAX>H` no debe desbordar ni entrar en pánico.
    saw_unit
        .then(|| hours.checked_mul(60)?.checked_add(minutes))
        .flatten()
}

/// Aplana `recipeInstructions` (cadena, lista, `HowToStep` o `HowToSection`)
/// a un texto único, en orden, uniendo con saltos de línea.
fn flatten_instructions(value: &Value) -> Option<String> {
    let mut parts = Vec::new();
    collect_instructions(value, &mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Acumula recursivamente el texto de las instrucciones.
fn collect_instructions(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => push_instruction(out, text),
        Value::Array(items) => {
            for item in items {
                collect_instructions(item, out);
            }
        }
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(Value::as_str) {
                push_instruction(out, text);
            }
            if let Some(items) = map.get("itemListElement") {
                collect_instructions(items, out);
            }
        }
        _ => {}
    }
}

/// Añade una instrucción no vacía, recortada.
fn push_instruction(out: &mut Vec<String>, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Envuelve un bloque JSON-LD en un HTML mínimo.
    fn html_with_json_ld(json: &str) -> String {
        format!(
            "<!doctype html><html><head>\
             <script type=\"application/ld+json\">{json}</script>\
             </head><body></body></html>"
        )
    }

    #[test]
    fn maps_root_recipe_fields() {
        let json = r#"{
            "@context": "https://schema.org",
            "@type": "Recipe",
            "name": "Tortilla de patatas",
            "url": "https://example.com/tortilla",
            "recipeYield": "4",
            "prepTime": "PT1H20M",
            "recipeIngredient": ["4 huevos", "3 patatas"],
            "recipeInstructions": ["Pelar y cortar", "Freír"]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta en la raíz");

        assert_eq!(recipe.name, "Tortilla de patatas");
        assert_eq!(
            recipe.source_url.as_deref(),
            Some("https://example.com/tortilla")
        );
        assert_eq!(recipe.servings, Some(4));
        assert_eq!(recipe.prep_time_minutes, Some(80));
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.ingredients[0].name, "4 huevos");
        assert_eq!(recipe.ingredients[0].unit, "ud");
        assert_eq!(recipe.ingredients[1].name, "3 patatas");
        assert_eq!(
            recipe.instructions.as_deref(),
            Some("Pelar y cortar\nFreír")
        );
    }

    #[test]
    fn finds_recipe_inside_graph() {
        let json = r#"{
            "@context": "https://schema.org",
            "@graph": [
                {"@type": "WebSite", "name": "Recetario"},
                {
                    "@type": "Recipe",
                    "name": "Gazpacho",
                    "recipeIngredient": ["Tomate"],
                    "url": "https://example.com/gazpacho"
                }
            ]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta en @graph");

        assert_eq!(recipe.name, "Gazpacho");
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(
            recipe.source_url.as_deref(),
            Some("https://example.com/gazpacho")
        );
    }

    #[test]
    fn missing_yield_defaults_to_two_servings() {
        let json = r#"{
            "@type": "Recipe",
            "name": "Sencilla",
            "recipeIngredient": ["Un ingrediente"]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta");

        assert_eq!(recipe.servings, Some(DEFAULT_SERVINGS));
    }

    #[test]
    fn missing_yield_defaults_and_converts_prep_and_total_times() {
        let json = r#"{
            "@type": "Recipe",
            "name": "Con tiempos",
            "recipeIngredient": ["Un ingrediente"],
            "prepTime": "PT1H20M",
            "totalTime": "PT1H45M"
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta");

        assert_eq!(recipe.servings, Some(DEFAULT_SERVINGS));
        assert_eq!(recipe.prep_time_minutes, Some(80));
    }

    #[test]
    fn total_time_used_when_prep_time_absent() {
        let json = r#"{
            "@type": "Recipe",
            "name": "Sin prep",
            "recipeIngredient": ["Un ingrediente"],
            "totalTime": "PT1H45M"
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta");

        assert_eq!(recipe.prep_time_minutes, Some(105));
    }

    #[test]
    fn flattens_nested_instructions_in_order() {
        let json = r#"{
            "@type": "Recipe",
            "name": "Anidada",
            "recipeIngredient": ["Un ingrediente"],
            "recipeInstructions": [
                {"@type": "HowToStep", "text": "Paso uno"},
                {
                    "@type": "HowToSection",
                    "name": "Relleno",
                    "itemListElement": [
                        {"@type": "HowToStep", "text": "Paso dos"},
                        {"@type": "HowToStep", "text": "Paso tres"}
                    ]
                },
                "Paso cuatro"
            ]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta");

        assert_eq!(
            recipe.instructions.as_deref(),
            Some("Paso uno\nPaso dos\nPaso tres\nPaso cuatro")
        );
    }

    #[test]
    fn html_without_recipe_returns_none() {
        let json = r#"{"@type": "WebPage", "name": "Sin receta"}"#;
        assert!(extract_recipe(&html_with_json_ld(json)).is_none());

        // HTML sin ningún bloque JSON-LD tampoco es una receta.
        assert!(extract_recipe("<html><body><h1>Hola</h1></body></html>").is_none());
    }

    #[test]
    fn recipe_type_as_array_is_recognized() {
        let json = r#"{
            "@type": ["Recipe", "Thing"],
            "name": "Array de tipos",
            "recipeIngredient": ["Un ingrediente"]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("recipe con @type array");

        assert_eq!(recipe.name, "Array de tipos");
    }

    #[test]
    fn finds_recipe_in_nested_graph() {
        let json = r#"{
            "@context": "https://schema.org",
            "@graph": [
                {"@type": "WebSite", "name": "Recetario"},
                {
                    "@graph": [
                        {"@type": "Recipe", "name": "Anidada", "recipeIngredient": ["Tomate"]}
                    ]
                }
            ]
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("recipe en @graph anidado");

        assert_eq!(recipe.name, "Anidada");
    }

    #[test]
    fn malformed_json_ld_is_ignored_without_panic() {
        // JSON inválido: el bloque se descarta y no hay receta.
        let html = html_with_json_ld("{ no es json }");
        assert!(extract_recipe(&html).is_none());

        // Un bloque inválido seguido de uno válido: se salta al válido.
        let html = "<html><head>\
             <script type=\"application/ld+json\">{ roto }</script>\
             <script type=\"application/ld+json\">\
             {\"@type\":\"Recipe\",\"name\":\"Buena\",\"recipeIngredient\":[\"x\"]}\
             </script></head></html>";
        let recipe = extract_recipe(html).expect("debe usar el bloque válido");
        assert_eq!(recipe.name, "Buena");
    }

    #[test]
    fn script_tag_with_multibyte_attribute_does_not_panic() {
        // Regresión: `&text[name_start + 1..]` partía a mitad de carácter UTF-8.
        for html in [
            "<html><head><script é>cuerpo</script></head></html>",
            "<html><head><script 😀>cuerpo</script></head></html>",
            "<html><head><script 日本語=\"x\">cuerpo</script></head></html>",
        ] {
            // No debe entrar en pánico; sin JSON-LD válido, no hay receta.
            assert!(extract_recipe(html).is_none());
        }
    }

    #[test]
    fn overflow_duration_does_not_panic() {
        let json = r#"{
            "@type": "Recipe",
            "name": "Duration enorme",
            "recipeIngredient": ["x"],
            "prepTime": "PT9223372036854775807H"
        }"#;

        let recipe = extract_recipe(&html_with_json_ld(json)).expect("receta");

        // `hours * 60` desborda i64: debe devolverse `None`, no paniquear ni envolver.
        assert_eq!(recipe.prep_time_minutes, None);
    }
}
