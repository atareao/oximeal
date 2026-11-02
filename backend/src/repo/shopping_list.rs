//! Repositorio de la lista de la compra (SQLite vía `sqlx`).

use anyhow::anyhow;
use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use crate::domain::category::{CATEGORY_ORDER, categorize};
use crate::domain::consolidate::ShoppingListItemDraft;
use crate::domain::ingredient::IngredientInput;
use crate::domain::normalize::normalize_name;
use crate::error::{AppError, AppResult};
use crate::models::shopping_list::{
    ShoppingItem, ShoppingItemInput, ShoppingItemPatch, ShoppingListGroup,
};

/// Proyección de un ítem, forzando texto en `updated_at`.
const SELECT_ITEM: &str = "SELECT id, name, amount, unit, category, checked, \
     CAST(updated_at AS TEXT) AS updated_at, client_id FROM shopping_list_items";

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
