//! Etiquetado automático de recetas.
//!
//! Vocabulario cerrado (D1), heurística determinista sobre los ingredientes
//! reforzada por menciones en el nombre y las instrucciones (D2), integración en
//! la transacción de escritura (D5) y backfill idempotente con marcador
//! persistente (D6).
//!
//! La normalización pasa el texto a minúsculas Unicode **sin diacríticos** y
//! compara por **token completo**: «panqueque» no casa con «pan».

use std::collections::HashSet;

use sqlx::{SqliteConnection, SqlitePool};

use crate::domain::ingredient::IngredientInput;
use crate::error::AppResult;
use crate::models::recipe::RecipeInput;
use crate::repo::tags;

/// Etiqueta del catálogo automático: `slug` canónico y nombre de presentación.
pub struct AutoTag {
    /// Identidad canónica normalizada.
    pub slug: &'static str,
    /// Nombre de presentación.
    pub name: &'static str,
}

/// Vocabulario cerrado de etiquetas automáticas (D1).
pub const AUTO_TAGS: &[AutoTag] = &[
    AutoTag {
        slug: "vegetariano",
        name: "Vegetariano",
    },
    AutoTag {
        slug: "vegano",
        name: "Vegano",
    },
    AutoTag {
        slug: "sin-gluten",
        name: "Sin gluten",
    },
    AutoTag {
        slug: "sin-lactosa",
        name: "Sin lactosa",
    },
    AutoTag {
        slug: "picante",
        name: "Picante",
    },
];

/// Versión del marcador de backfill (`PRAGMA user_version`).
pub const BACKFILL_VERSION: i64 = 1;

/// Carne y pescado: vetados para `vegetariano` y `vegano`.
const MEAT_FISH: &[&str] = &[
    "carne",
    "pollo",
    "pavo",
    "ternera",
    "cerdo",
    "cordero",
    "conejo",
    "pato",
    "jamon",
    "chorizo",
    "salchicha",
    "bacon",
    "beicon",
    "pescado",
    "atun",
    "salmon",
    "merluza",
    "bacalao",
    "trucha",
    "lubina",
    "sardina",
    "anchoa",
    "gamba",
    "gambas",
    "marisco",
    "calamar",
    "pulpo",
    "mejillon",
    "almeja",
    "cangrejo",
];

/// Productos de origen animal adicionales: vetados solo para `vegano`.
const ANIMAL: &[&str] = &[
    "huevo",
    "huevos",
    "leche",
    "queso",
    "yogur",
    "mantequilla",
    "nata",
    "crema",
    "miel",
    "requeson",
    "mozzarella",
    "parmesano",
    "bechamel",
    "lactosa",
];

/// Lácteos: vetados para `sin-lactosa`.
const LACTOSE: &[&str] = &[
    "leche",
    "queso",
    "yogur",
    "mantequilla",
    "nata",
    "crema",
    "requeson",
    "mozzarella",
    "parmesano",
    "bechamel",
    "lactosa",
];

/// Cereales y derivados con gluten: vetados para `sin-gluten`.
const GLUTEN: &[&str] = &[
    "trigo",
    "harina",
    "pan",
    "pasta",
    "cebada",
    "centeno",
    "semola",
    "fideo",
    "fideos",
    "espagueti",
    "spaghetti",
    "macarron",
    "macarrones",
    "galleta",
    "cerveza",
    "cuscus",
    "ramen",
    "empanada",
    "bolleria",
];

/// Términos picantes: emitidos por presencia.
const SPICY: &[&str] = &[
    "picante",
    "chile",
    "guindilla",
    "jalapeno",
    "tabasco",
    "cayena",
    "sriracha",
    "wasabi",
    "habanero",
];

/// Devuelve el `slug` del catálogo automático al que corresponde `raw`, si lo hay.
///
/// Compara por una clave normalizada (minúsculas sin diacríticos, separadores
/// colapsados a `-`), de modo que «Sin gluten» y «sin-gluten» resuelven a la
/// misma etiqueta canónica.
pub fn canonical_auto_slug(raw: &str) -> Option<&'static str> {
    let key = auto_key(raw);
    AUTO_TAGS
        .iter()
        .find(|tag| tag.slug == key)
        .map(|tag| tag.slug)
}

/// Nombre de presentación del catálogo automático para un `slug` dado.
pub fn name_for_slug(slug: &str) -> Option<&'static str> {
    AUTO_TAGS
        .iter()
        .find(|tag| tag.slug == slug)
        .map(|tag| tag.name)
}

/// Aplica el etiquetado automático a una receta dentro de la transacción dada.
///
/// Persiste las supresiones enviadas (la manual gana) y recalcula las
/// asociaciones `auto`; respeta `auto_tags: false`.
pub(crate) async fn apply(
    conn: &mut SqliteConnection,
    recipe_id: &str,
    input: &RecipeInput,
) -> AppResult<()> {
    tags::replace_suppressions(conn, recipe_id, &input.suppressed_auto_tags).await?;
    recalculate(
        conn,
        recipe_id,
        &input.ingredients,
        &input.name,
        input.instructions.as_deref(),
        input.auto_tags,
    )
    .await
}

/// Recalcula las etiquetas `auto` de una receta a partir de su contenido.
///
/// Borra las asociaciones `auto` previas y reinserta las inferidas que no estén
/// suprimidas ni ya fijadas como `manual`.
pub(crate) async fn recalculate(
    conn: &mut SqliteConnection,
    recipe_id: &str,
    ingredients: &[IngredientInput],
    name: &str,
    instructions: Option<&str>,
    enabled: bool,
) -> AppResult<()> {
    sqlx::query("DELETE FROM recipe_tags WHERE recipe_id = ? AND source = 'auto'")
        .bind(recipe_id)
        .execute(&mut *conn)
        .await?;

    if !enabled {
        return Ok(());
    }

    let inferred = infer(ingredients, name, instructions);
    if inferred.is_empty() {
        return Ok(());
    }

    let suppressed: HashSet<String> = sqlx::query_scalar(
        "SELECT t.slug FROM recipe_tag_suppressions s \
         JOIN tags t ON t.id = s.tag_id WHERE s.recipe_id = ?",
    )
    .bind(recipe_id)
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .collect();

    let manual: HashSet<String> = sqlx::query_scalar(
        "SELECT t.slug FROM recipe_tags rt \
         JOIN tags t ON t.id = rt.tag_id \
         WHERE rt.recipe_id = ? AND rt.source = 'manual'",
    )
    .bind(recipe_id)
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .collect();

    for slug in inferred {
        if suppressed.contains(slug) || manual.contains(slug) {
            continue;
        }
        let tag_id = tags::upsert_auto(conn, slug).await?;
        sqlx::query(
            "INSERT INTO recipe_tags (recipe_id, tag_id, source) VALUES (?, ?, 'auto') \
             ON CONFLICT(recipe_id, tag_id) DO NOTHING",
        )
        .bind(recipe_id)
        .bind(&tag_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

/// Etiquetado automático idempotente de las recetas existentes (D6).
///
/// Recorre las recetas sin asociaciones `auto` y les aplica la heurística. Usa
/// `PRAGMA user_version` como marcador persistente: solo se ejecuta una vez por
/// base de datos. Si alguna fila no se puede procesar, el marcador NO se avanza
/// para que esas recetas se reintenten en un arranque posterior.
pub async fn backfill(pool: &SqlitePool) -> AppResult<()> {
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    if version >= BACKFILL_VERSION {
        return Ok(());
    }

    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, CAST(ingredients AS TEXT), instructions FROM recipes r \
         WHERE NOT EXISTS ( \
             SELECT 1 FROM recipe_tags rt WHERE rt.recipe_id = r.id AND rt.source = 'auto' \
         )",
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut failed: u32 = 0;

    for (id, name, ingredients_json, instructions) in rows {
        // Política tolerante: una fila con `ingredients` corrupto no debe
        // abortar el arranque. Se registra el error y se continúa con el resto.
        let ingredients: Vec<IngredientInput> = match serde_json::from_str(&ingredients_json) {
            Ok(ingredients) => ingredients,
            Err(error) => {
                tracing::error!(
                    recipe_id = %id,
                    error = %error,
                    "ingredientes JSON inválidos; se omite la receta en el backfill"
                );
                failed += 1;
                continue;
            }
        };

        if let Err(error) = recalculate(
            &mut tx,
            &id,
            &ingredients,
            &name,
            instructions.as_deref(),
            true,
        )
        .await
        {
            tracing::error!(
                recipe_id = %id,
                error = %error,
                "falló el recálculo de etiquetas; se omite la receta en el backfill"
            );
            failed += 1;
            continue;
        }
    }

    tx.commit().await?;

    // Si hubo fallos, no se avanza el marcador: el siguiente arranque volverá a
    // intentar las recetas pendientes (comportamiento esperado en `main.rs`).
    if failed > 0 {
        tracing::warn!(
            failed,
            "el backfill de etiquetas automáticas dejó recetas pendientes; \
             no se avanza el marcador y se reintentarán en otro arranque"
        );
        return Ok(());
    }

    // El valor es una constante de código, no entrada del usuario.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "PRAGMA user_version = {BACKFILL_VERSION}"
    )))
    .execute(pool)
    .await?;

    Ok(())
}

/// Infiere las etiquetas automáticas de una receta (determinista, sin red).
fn infer(
    ingredients: &[IngredientInput],
    name: &str,
    instructions: Option<&str>,
) -> Vec<&'static str> {
    let mut tokens: HashSet<String> = HashSet::new();
    for ingredient in ingredients {
        add_tokens(&ingredient.name, &mut tokens);
    }
    add_tokens(name, &mut tokens);
    if let Some(text) = instructions {
        add_tokens(text, &mut tokens);
    }

    let has = |list: &[&str]| list.iter().any(|term| tokens.contains(*term));

    let has_meat = has(MEAT_FISH);
    let has_animal = has(ANIMAL);

    let mut result = Vec::new();
    if !has_meat {
        result.push("vegetariano");
    }
    if !has_meat && !has_animal {
        result.push("vegano");
    }
    if !has(GLUTEN) {
        result.push("sin-gluten");
    }
    if !has(LACTOSE) {
        result.push("sin-lactosa");
    }
    if has(SPICY) {
        result.push("picante");
    }
    result
}

/// Añade a `set` los tokens alfanuméricos de `text` normalizados.
fn add_tokens(text: &str, set: &mut HashSet<String>) {
    let normalized = strip_diacritics(&text.to_lowercase());
    for token in normalized.split(|c: char| !c.is_alphanumeric()) {
        if !token.is_empty() {
            set.insert(token.to_string());
        }
    }
}

/// Clave de comparación con el catálogo: minúsculas, sin diacríticos y con los
/// separadores colapsados a `-`.
fn auto_key(raw: &str) -> String {
    let normalized = strip_diacritics(&raw.to_lowercase());
    let mut out = String::new();
    let mut prev_sep = false;
    for c in normalized.chars() {
        if c.is_alphanumeric() {
            out.push(c);
            prev_sep = false;
        } else if !out.is_empty() && !prev_sep {
            out.push('-');
            prev_sep = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// Pasa `text` a minúsculas sin diacríticos (solo lo necesario para el español).
fn strip_diacritics(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ingredient(name: &str) -> IngredientInput {
        IngredientInput {
            name: name.to_string(),
            amount: 1.0,
            unit: "ud".to_string(),
            category: None,
        }
    }

    #[test]
    fn vegan_by_absence_of_animal_origin() {
        let result = infer(
            &[ingredient("Tomate"), ingredient("Lechuga")],
            "Ensalada",
            None,
        );
        assert!(result.contains(&"vegano"));
        assert!(result.contains(&"vegetariano"));
        assert!(result.contains(&"sin-gluten"));
    }

    #[test]
    fn meat_excludes_vegetarian_and_vegan() {
        let result = infer(
            &[ingredient("Pollo"), ingredient("Arroz")],
            "Pollo con arroz",
            None,
        );
        assert!(!result.contains(&"vegetariano"));
        assert!(!result.contains(&"vegano"));
        assert!(result.contains(&"sin-gluten"));
    }

    #[test]
    fn egg_excludes_vegan_but_not_vegetarian() {
        let result = infer(&[ingredient("Huevo")], "Tortilla", None);
        assert!(result.contains(&"vegetariano"));
        assert!(!result.contains(&"vegano"));
    }

    #[test]
    fn spicy_by_presence() {
        let result = infer(&[ingredient("Chile")], "Salsa", None);
        assert!(result.contains(&"picante"));
    }

    #[test]
    fn token_match_not_substring() {
        let result = infer(&[ingredient("Panqueque")], "Panqueques", None);
        assert!(result.contains(&"sin-gluten"));
    }

    #[test]
    fn canonical_auto_slug_matches_names_and_slugs() {
        assert_eq!(canonical_auto_slug("Sin gluten"), Some("sin-gluten"));
        assert_eq!(canonical_auto_slug("sin-gluten"), Some("sin-gluten"));
        assert_eq!(canonical_auto_slug("Vegano"), Some("vegano"));
        assert_eq!(canonical_auto_slug("Sin uso"), None);
    }
}
