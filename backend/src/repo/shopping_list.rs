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
     CAST(updated_at AS TEXT) AS updated_at FROM shopping_list_items";

/// Devuelve la lista completa agrupada por categoría, en orden de pasillo.
pub async fn list_grouped(pool: &SqlitePool) -> AppResult<Vec<ShoppingListGroup>> {
    let sql = format!("{SELECT_ITEM} ORDER BY name COLLATE NOCASE ASC, id ASC");
    let items: Vec<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await?;
    Ok(group_items(items))
}

/// Crea un ítem manual (sin `source_plan_id`).
///
/// Si no se indica categoría, se infiere con la heurística de dominio sobre el
/// nombre normalizado.
pub async fn create_item(pool: &SqlitePool, input: &ShoppingItemInput) -> AppResult<ShoppingItem> {
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
    sqlx::query(
        "INSERT INTO shopping_list_items \
         (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
         VALUES (?, ?, ?, ?, ?, 0, NULL, CURRENT_TIMESTAMP)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(input.amount)
    .bind(&input.unit)
    .bind(&category)
    .execute(pool)
    .await?;

    get_item(pool, &id).await
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
    let mut tx = pool.begin().await?;

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
    let ingredient_blobs: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(r.ingredients AS TEXT) FROM meal_plan_entries e \
         JOIN recipes r ON r.id = e.recipe_id \
         WHERE e.date >= ? AND e.date <= ?",
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(&mut *conn)
    .await?;

    let mut inputs: Vec<IngredientInput> = Vec::new();
    for blob in ingredient_blobs {
        let parsed: Vec<IngredientInput> = serde_json::from_str(&blob)
            .map_err(|error| AppError::Internal(anyhow!("ingredientes JSON inválidos: {error}")))?;
        inputs.extend(parsed);
    }

    Ok(crate::domain::consolidate::consolidate(&inputs))
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
