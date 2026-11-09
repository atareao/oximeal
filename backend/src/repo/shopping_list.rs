//! Repositorio de la lista de la compra (SQLite vía `sqlx`).

use anyhow::anyhow;
use sqlx::{SqliteConnection, SqlitePool};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use crate::domain::category::{CATEGORY_ORDER, categorize};
use crate::domain::consolidate::{ShoppingListItemDraft, round2};
use crate::domain::date::parse_iso_date;
use crate::domain::ingredient::IngredientInput;
use crate::domain::normalize::normalize_name;
use crate::domain::units::{UnitFamily, canonicalize, normalize_unit};
use crate::error::{AppError, AppResult};
use crate::models::shopping_list::{
    ShoppingItem, ShoppingItemInput, ShoppingItemPatch, ShoppingListGroup,
};

/// Proyección de un ítem, forzando texto en `updated_at`.
const SELECT_ITEM: &str = "SELECT id, name, amount, unit, category, checked, \
     CAST(updated_at AS TEXT) AS updated_at, client_id FROM shopping_list_items";

/// Valor de `source_recipe_id` para las líneas que fusionan varias recetas.
///
/// Una línea consolidada a partir de un mismo ingrediente aportado por más de
/// una receta ya no tiene una atribución unívoca; se marca con este centinela.
const SOURCE_CONSOLIDATED: &str = "consolidated";

/// Borrador derivado de recetas junto con su receta fuente.
///
/// `source_recipe_id` **nunca** es nulo: o bien es el id de la única receta que
/// aporta el ingrediente, o bien [`SOURCE_CONSOLIDATED`] si lo aportan varias.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecipeDerivedDraft {
    /// Línea consolidada lista para insertar en la lista.
    pub draft: ShoppingListItemDraft,
    /// Procedencia de la línea: id de receta o [`SOURCE_CONSOLIDATED`].
    pub source_recipe_id: String,
}

/// Resultado del alta de un ítem: nuevo o ya existente por `client_id`.
///
/// La ruta usa esta distinción para responder `201` al crear y `200` cuando el
/// alta idempotente devuelve un ítem que ya existía.
#[derive(Debug, Clone, PartialEq)]
pub enum CreateOutcome {
    /// El ítem se ha insertado en esta petición.
    Created(ShoppingItem),
    /// Ya existía un ítem con el mismo `client_id`; se devuelve sin insertar.
    Existing(ShoppingItem),
}

impl CreateOutcome {
    /// Devuelve el ítem, sea nuevo o existente.
    pub fn into_item(self) -> ShoppingItem {
        match self {
            CreateOutcome::Created(item) | CreateOutcome::Existing(item) => item,
        }
    }
}

/// Devuelve la lista completa agrupada por categoría, en orden de pasillo.
pub async fn list_grouped(pool: &SqlitePool) -> AppResult<Vec<ShoppingListGroup>> {
    let sql = format!("{SELECT_ITEM} ORDER BY name COLLATE NOCASE ASC, id ASC");
    let items: Vec<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await?;
    Ok(group_items(items))
}

/// Crea un ítem manual (sin `source_plan_id`), de forma idempotente por `client_id`.
///
/// Si no se indica categoría, se infiere con la heurística de dominio sobre el
/// nombre normalizado.
///
/// Cuando `input.client_id` trae un valor no vacío:
/// 1. Si ya existe un ítem con ese `client_id`, se devuelve como
///    [`CreateOutcome::Existing`] sin insertar.
/// 2. Si no existe, se inserta con `client_id` y se devuelve como
///    [`CreateOutcome::Created`].
/// 3. Si el `INSERT` choca con el índice único parcial (carrera entre dos
///    peticiones con el mismo `client_id`), se relee el ítem ganador y se
///    devuelve como [`CreateOutcome::Existing`].
pub async fn create_item(pool: &SqlitePool, input: &ShoppingItemInput) -> AppResult<CreateOutcome> {
    let client_id = input
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(client_id) = client_id
        && let Some(existing) = find_by_client_id(pool, client_id).await?
    {
        return Ok(CreateOutcome::Existing(existing));
    }

    let category = match input
        .category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(category) => category.to_string(),
        None => categorize(&normalize_name(&input.name)).to_string(),
    };

    let id = Uuid::new_v4().to_string();
    let result = sqlx::query(
        "INSERT INTO shopping_list_items \
         (id, name, amount, unit, category, checked, source_plan_id, updated_at, client_id) \
         VALUES (?, ?, ?, ?, ?, 0, NULL, CURRENT_TIMESTAMP, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(input.amount)
    .bind(&input.unit)
    .bind(&category)
    .bind(client_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(CreateOutcome::Created(get_item(pool, &id).await?)),
        Err(error) if is_unique_violation(&error) => {
            // Otra petición ganó la carrera: su ítem es el canónico.
            match client_id {
                Some(client_id) => match find_by_client_id(pool, client_id).await? {
                    Some(existing) => Ok(CreateOutcome::Existing(existing)),
                    None => Err(error.into()),
                },
                None => Err(error.into()),
            }
        }
        Err(error) => Err(error.into()),
    }
}

/// Busca un ítem por `client_id`; `None` si no existe.
async fn find_by_client_id(pool: &SqlitePool, client_id: &str) -> AppResult<Option<ShoppingItem>> {
    let sql = format!("{SELECT_ITEM} WHERE client_id = ?");
    let item: Option<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(client_id)
        .fetch_optional(pool)
        .await?;
    Ok(item)
}

/// Indica si el error es una violación de restricción única.
fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}

/// Obtiene un ítem por id; 404 si no existe.
pub async fn get_item(pool: &SqlitePool, id: &str) -> AppResult<ShoppingItem> {
    let sql = format!("{SELECT_ITEM} WHERE id = ?");
    let item: Option<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(pool)
        .await?;
    item.ok_or_else(|| AppError::NotFound(format!("ítem de la compra {id}")))
}

/// Actualiza parcialmente un ítem (`checked` y/o `amount`) refrescando `updated_at`.
pub async fn patch_item(
    pool: &SqlitePool,
    id: &str,
    patch: &ShoppingItemPatch,
) -> AppResult<ShoppingItem> {
    // Garantiza 404 si el ítem no existe antes de intentar actualizar.
    get_item(pool, id).await?;

    if let Some(checked) = patch.checked {
        sqlx::query(
            "UPDATE shopping_list_items SET checked = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(checked)
        .bind(id)
        .execute(pool)
        .await?;
    }

    if let Some(amount) = patch.amount {
        sqlx::query(
            "UPDATE shopping_list_items SET amount = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(amount)
        .bind(id)
        .execute(pool)
        .await?;
    }

    if let Some(category) = patch.category.as_deref() {
        sqlx::query(
            "UPDATE shopping_list_items SET category = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(category)
        .bind(id)
        .execute(pool)
        .await?;
    }

    get_item(pool, id).await
}

/// Regenera los ítems derivados del plan para un rango de fechas.
///
/// **Semántica de la lista de la compra**: existe una **única lista semanal
/// global**; no hay una lista por rango ni por plan. Por eso los ítems derivados
/// del plan se identifican con `source_plan_id` no nulo y se reemplazan **todos**
/// (no solo los del rango indicado) al reconstruir. El rango aquí solo determina
/// qué entradas del plan se leen para calcular las cantidades.
///
/// Lee las recetas asignadas en el rango, consolida sus ingredientes con el
/// motor de dominio y reemplaza los ítems con `source_plan_id` no nulo. Los
/// ítems manuales (`source_plan_id` nulo) se conservan. Toda la regeneración
/// ocurre en una única transacción.
pub async fn consolidate(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<ShoppingListGroup>> {
    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let drafts = collect_drafts(&mut tx, start_date, end_date).await?;
    replace_consolidated(&mut tx, &drafts).await?;

    tx.commit().await?;

    list_grouped(pool).await
}

/// Vuelca los ingredientes de una receta a la lista de la compra.
///
/// Registra (o actualiza) la receta y sus raciones en `shopping_recipe_sources`
/// y reconstruye **todas** las líneas derivadas de recetas reconsolidando las
/// fuentes registradas. Es decir, el volcado es **declarativo**: repetir la
/// petición no duplica y cambiar las raciones reescala sin acumular. Las líneas
/// derivadas del plan (`source_plan_id` no nulo) y los ítems manuales se
/// conservan intactos.
///
/// Si la receta no existe, devuelve [`AppError::NotFound`] sin escribir nada; si
/// las raciones solicitadas son `< 1`, devuelve [`AppError::Validation`] también
/// sin escribir. Toda la operación ocurre en una única transacción
/// `BEGIN IMMEDIATE` (escritura lee-luego-escribe en WAL).
pub async fn from_recipe(
    pool: &SqlitePool,
    recipe_id: &str,
    servings: Option<i64>,
) -> AppResult<Vec<ShoppingListGroup>> {
    // `BEGIN IMMEDIATE` evita SQLITE_BUSY_SNAPSHOT en el patrón leer-luego-escribir.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let recipe_servings: Option<i64> =
        sqlx::query_scalar("SELECT COALESCE(servings, 2) FROM recipes WHERE id = ?")
            .bind(recipe_id)
            .fetch_optional(&mut *tx)
            .await?;

    let Some(recipe_servings) = recipe_servings else {
        // La transacción se descarta (rollback implícito): no se escribe nada.
        return Err(AppError::NotFound(format!("receta {recipe_id}")));
    };

    let requested = servings.unwrap_or(recipe_servings);
    if requested < 1 {
        return Err(AppError::Validation("servings debe ser >= 1".into()));
    }

    sqlx::query(
        "INSERT INTO shopping_recipe_sources (recipe_id, servings) VALUES (?, ?) \
         ON CONFLICT(recipe_id) DO UPDATE SET servings = excluded.servings",
    )
    .bind(recipe_id)
    .bind(requested)
    .execute(&mut *tx)
    .await?;

    let drafts = collect_recipe_drafts(&mut tx).await?;
    replace_recipe_derived(&mut tx, &drafts).await?;

    tx.commit().await?;

    list_grouped(pool).await
}

/// Objetivo de fusión de una línea existente: id, cantidad canónica y unidad.
#[derive(Debug, Clone)]
struct MergeTarget {
    id: String,
    amount: f64,
    unit: String,
}

/// Valida que el rango sea parseable y que `start_date <= end_date`.
///
/// Defensa en profundidad: los handlers ya validan el rango, pero el repositorio
/// no debe aceptar un rango invertido ni fechas inválidas si se invoca desde
/// otro punto de la aplicación.
fn validate_range(start_date: &str, end_date: &str) -> AppResult<()> {
    let start = parse_iso_date(start_date)
        .ok_or_else(|| AppError::Validation(format!("fecha inválida: {start_date}")))?;
    let end = parse_iso_date(end_date)
        .ok_or_else(|| AppError::Validation(format!("fecha inválida: {end_date}")))?;
    if start > end {
        return Err(AppError::Validation(
            "start_date no puede ser posterior a end_date".into(),
        ));
    }
    Ok(())
}

/// Añade los ingredientes del plan en un rango a la lista global, fusionando.
///
/// A diferencia de [`consolidate`], que **reemplaza** las líneas del plan, este
/// volcado es **aditivo**: los ingredientes del plan se fusionan con los ítems
/// existentes por nombre normalizado y unidad canónica. Si coincide con un ítem
/// existente, se suma su cantidad; si no, se inserta un ítem nuevo. Los ítems
/// manuales, los volcados de receta y los del plan se conservan junto con sus
/// `checked`/`client_id`. Toda la operación ocurre en una transacción
/// `BEGIN IMMEDIATE`.
pub async fn from_plan(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<ShoppingListGroup>> {
    validate_range(start_date, end_date)?;

    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let drafts = collect_drafts(&mut tx, start_date, end_date).await?;
    merge_drafts(&mut tx, &drafts).await?;

    tx.commit().await?;

    list_grouped(pool).await
}

/// Fusiona los borradores del plan con los ítems existentes, sumando cantidades.
///
/// La clave de conciliación es `(nombre normalizado, familia, unidad canónica
/// normalizada)`, el mismo subgrupo que usa el motor de dominio. La unidad se
/// normaliza con [`normalize_unit`] para que las variantes de `Unknown` (p. ej.
/// `"Cajas"` y `"cajas"`) coincidan. Si ya existe una línea con esa clave, se
/// suma su cantidad canónica y se conserva la línea (y su `checked`, `client_id`,
/// `source_plan_id` y `source_recipe_id`); si no, se inserta una línea nueva con
/// `source_plan_id` y `source_recipe_id` nulos.
///
/// Usa `conn` sin abrir transacción propia.
pub(crate) async fn merge_drafts(
    conn: &mut SqliteConnection,
    drafts: &[ShoppingListItemDraft],
) -> AppResult<()> {
    let sql = SELECT_ITEM.to_string();
    let items: Vec<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(&mut *conn)
        .await?;

    let mut targets: HashMap<(String, UnitFamily, String), MergeTarget> = HashMap::new();
    for item in &items {
        let canonical = canonicalize(0.0, &item.unit);
        // La clave normaliza la unidad para que, en la familia `Unknown`, las
        // variantes de mayúsculas/espacios (`"Cajas"` y `"cajas"`) se fusionen.
        let key = (
            normalize_name(&item.name),
            canonical.family,
            normalize_unit(&canonical.display_unit),
        );
        targets.entry(key).or_insert_with(|| MergeTarget {
            id: item.id.clone(),
            amount: canonicalize(item.amount, &item.unit).amount,
            unit: canonical.display_unit,
        });
    }

    for draft in drafts {
        let canonical = canonicalize(draft.amount, &draft.unit);
        let key = (
            normalize_name(&draft.name),
            canonical.family,
            normalize_unit(&canonical.display_unit),
        );

        if let Some(target) = targets.get_mut(&key) {
            // Un aporte de 0.0 (p. ej. `al gusto`) no cambia la fila: no se
            // refresca `updated_at` ni se reescribe la unidad.
            if canonical.amount == 0.0 {
                continue;
            }
            let total = round2(target.amount + canonical.amount);
            target.amount = total;
            sqlx::query(
                "UPDATE shopping_list_items SET amount = ?, unit = ?, \
                 updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(total)
            .bind(&target.unit)
            .bind(&target.id)
            .execute(&mut *conn)
            .await?;
        } else {
            let id = Uuid::new_v4().to_string();
            let amount = round2(canonical.amount);
            sqlx::query(
                "INSERT INTO shopping_list_items \
                 (id, name, amount, unit, category, checked, source_plan_id, source_recipe_id, updated_at) \
                 VALUES (?, ?, ?, ?, ?, 0, NULL, NULL, CURRENT_TIMESTAMP)",
            )
            .bind(&id)
            .bind(&draft.name)
            .bind(amount)
            .bind(&canonical.display_unit)
            .bind(&draft.category)
            .execute(&mut *conn)
            .await?;
            targets.insert(
                key,
                MergeTarget {
                    id,
                    amount,
                    unit: canonical.display_unit,
                },
            );
        }
    }

    Ok(())
}

/// Recolecta los borradores consolidados de todas las recetas volcadas.
///
/// Lee cada fuente registrada (`shopping_recipe_sources`), escala sus
/// ingredientes por el factor raciones_solicitadas / raciones_receta y consolida
/// el conjunto con el motor de dominio. Para cada línea resultante, resuelve su
/// `source_recipe_id`: el id de la receta cuando un único origen aporta ese
/// ingrediente, o [`SOURCE_CONSOLIDATED`] cuando lo aportan varias.
///
/// La atribución se indexa por **(nombre normalizado, familia, unidad canónica)**,
/// el mismo subgrupo con el que el motor `consolidate` subdivide un ingrediente:
/// así una misma leche en `ml` y en `ud` produce dos líneas y cada una conserva la
/// receta que la aporta, en vez de marcarse ambas como consolidadas.
///
/// No abre transacción propia: el llamante controla la atomicidad.
pub(crate) async fn collect_recipe_drafts(
    conn: &mut SqliteConnection,
) -> AppResult<Vec<RecipeDerivedDraft>> {
    let rows: Vec<(String, i64, i64, String)> = sqlx::query_as(
        "SELECT s.recipe_id, s.servings, COALESCE(r.servings, 2), CAST(r.ingredients AS TEXT) \
         FROM shopping_recipe_sources s \
         JOIN recipes r ON r.id = s.recipe_id",
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut inputs: Vec<IngredientInput> = Vec::new();
    // (nombre normalizado, familia, unidad canónica) -> recetas que lo aportan.
    let mut attribution: HashMap<(String, UnitFamily, String), BTreeSet<String>> = HashMap::new();

    for (recipe_id, source_servings, recipe_servings, blob) in rows {
        let factor = servings_factor(source_servings, recipe_servings);
        let parsed: Vec<IngredientInput> = serde_json::from_str(&blob)
            .map_err(|error| AppError::Internal(anyhow!("ingredientes JSON inválidos: {error}")))?;

        for mut ingredient in parsed {
            ingredient.amount *= factor;
            let canon = canonicalize(ingredient.amount, &ingredient.unit);
            attribution
                .entry((
                    normalize_name(&ingredient.name),
                    canon.family,
                    canon.display_unit,
                ))
                .or_default()
                .insert(recipe_id.clone());
            inputs.push(ingredient);
        }
    }

    let drafts = crate::domain::consolidate::consolidate(&inputs);

    Ok(drafts
        .into_iter()
        .map(|draft| {
            let canon = canonicalize(draft.amount, &draft.unit);
            let key = (
                normalize_name(&draft.name),
                canon.family,
                canon.display_unit,
            );
            let source_recipe_id = match attribution.get(&key) {
                Some(origins) => {
                    let mut iter = origins.iter();
                    match (iter.next(), iter.next()) {
                        (Some(single), None) => single.clone(),
                        _ => SOURCE_CONSOLIDATED.to_string(),
                    }
                }
                None => SOURCE_CONSOLIDATED.to_string(),
            };
            RecipeDerivedDraft {
                draft,
                source_recipe_id,
            }
        })
        .collect())
}

/// Reemplaza las líneas derivadas de recetas (`source_recipe_id` no nulo) por los
/// borradores dados, conservando los ítems manuales y las líneas del plan. Usa
/// `conn` sin abrir transacción propia.
///
/// **Nota**: las líneas derivadas de recetas se recrean siempre con `checked = 0`,
/// de modo que repetir el volcado resetea el estado "ya comprado" de esas líneas.
/// Los ítems manuales y las líneas del plan no se tocan y conservan su `checked`.
pub(crate) async fn replace_recipe_derived(
    conn: &mut SqliteConnection,
    drafts: &[RecipeDerivedDraft],
) -> AppResult<()> {
    sqlx::query("DELETE FROM shopping_list_items WHERE source_recipe_id IS NOT NULL")
        .execute(&mut *conn)
        .await?;

    for entry in drafts {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO shopping_list_items \
             (id, name, amount, unit, category, checked, source_plan_id, source_recipe_id, updated_at) \
             VALUES (?, ?, ?, ?, ?, 0, NULL, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&id)
        .bind(&entry.draft.name)
        .bind(entry.draft.amount)
        .bind(&entry.draft.unit)
        .bind(&entry.draft.category)
        .bind(&entry.source_recipe_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

/// Recolecta los borradores consolidados del plan en un rango, usando `conn`.
///
/// No abre transacción propia: el llamante controla la atomicidad y puede
/// encadenarla con otras escrituras sobre la misma conexión.
pub(crate) async fn collect_drafts(
    conn: &mut SqliteConnection,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<ShoppingListItemDraft>> {
    // Se leen las raciones planificadas y las de la receta para escalar las
    // cantidades por el factor `entry / recipe` (ambas con valor por defecto 2).
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT CAST(r.ingredients AS TEXT), COALESCE(e.servings, 2), COALESCE(r.servings, 2) \
         FROM meal_plan_entries e \
         JOIN recipes r ON r.id = e.recipe_id \
         WHERE e.date >= ? AND e.date <= ?",
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(&mut *conn)
    .await?;

    let mut inputs: Vec<IngredientInput> = Vec::new();
    for (blob, entry_servings, recipe_servings) in rows {
        let factor = servings_factor(entry_servings, recipe_servings);
        let parsed: Vec<IngredientInput> = serde_json::from_str(&blob)
            .map_err(|error| AppError::Internal(anyhow!("ingredientes JSON inválidos: {error}")))?;
        inputs.extend(parsed.into_iter().map(|mut ingredient| {
            ingredient.amount *= factor;
            ingredient
        }));
    }

    Ok(crate::domain::consolidate::consolidate(&inputs))
}

/// Factor de escalado de una entrada del plan: raciones planificadas / raciones de la receta.
///
/// El esquema garantiza `servings >= 1` (o `NULL`→2); el `max(1)` acota además
/// el denominador por defensa ante datos heredados que pudieran traer `0`.
fn servings_factor(entry_servings: i64, recipe_servings: i64) -> f64 {
    // El esquema garantiza `servings >= 1`, pero se acota por defensa ante datos
    // heredados: un denominador 0 produciría un `amount` infinito.
    entry_servings as f64 / recipe_servings.max(1) as f64
}

/// Reemplaza los ítems derivados del plan (`source_plan_id` no nulo) por los
/// borradores dados, conservando los ítems manuales. Usa `conn` sin abrir
/// transacción propia.
///
/// El borrado `WHERE source_plan_id IS NOT NULL` **no** se acota al rango de
/// fechas a propósito: la lista de la compra es una única lista semanal global,
/// así que cada reconstrucción parte de cero para los ítems del plan.
pub(crate) async fn replace_consolidated(
    conn: &mut SqliteConnection,
    drafts: &[ShoppingListItemDraft],
) -> AppResult<()> {
    sqlx::query("DELETE FROM shopping_list_items WHERE source_plan_id IS NOT NULL")
        .execute(&mut *conn)
        .await?;

    for draft in drafts {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO shopping_list_items \
             (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
             VALUES (?, ?, ?, ?, ?, 0, 'consolidated', CURRENT_TIMESTAMP)",
        )
        .bind(&id)
        .bind(&draft.name)
        .bind(draft.amount)
        .bind(&draft.unit)
        .bind(&draft.category)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

/// Elimina los ítems marcados como comprados.
pub async fn delete_completed(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query("DELETE FROM shopping_list_items WHERE checked = 1")
        .execute(pool)
        .await?;
    Ok(())
}

/// Agrupa los ítems por categoría respetando el orden de pasillo.
///
/// Las categorías desconocidas se incluyen en `"Otros"`.
fn group_items(items: Vec<ShoppingItem>) -> Vec<ShoppingListGroup> {
    let mut groups = Vec::new();

    for category in CATEGORY_ORDER {
        let bucket: Vec<ShoppingItem> = items
            .iter()
            .filter(|item| {
                if category == "Otros" {
                    item.category == "Otros" || !CATEGORY_ORDER.contains(&item.category.as_str())
                } else {
                    item.category == category
                }
            })
            .cloned()
            .collect();

        if !bucket.is_empty() {
            groups.push(ShoppingListGroup {
                category: category.to_string(),
                items: bucket,
            });
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::servings_factor;

    #[test]
    fn servings_factor_es_ratio_de_raciones() {
        assert_eq!(servings_factor(4, 2), 2.0);
        assert_eq!(servings_factor(2, 4), 0.5);
        assert_eq!(servings_factor(2, 2), 1.0);
    }

    #[test]
    fn servings_factor_defiende_denominador_cero() {
        assert_eq!(servings_factor(4, 0), 4.0);
    }
}
