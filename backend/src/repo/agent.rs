//! Repositorio de la Agent API (SQLite vía `sqlx`).
//!
//! Agrupa las operaciones que el asistente Valet necesita: recetas recientes,
//! fijado transaccional del plan semanal y gestión de la lista de la compra.

use std::collections::HashMap;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::category::categorize;
use crate::domain::consolidate::round2;
use crate::domain::date::canonical_iso_date;
use crate::domain::ingredient::IngredientInput;
use crate::domain::normalize::normalize_name;
use crate::domain::units::{canonicalize, normalize_unit};
use crate::error::{AppError, AppResult};
use crate::models::agent::{RecentRecipe, StructuredPlanEntry};
use crate::models::shopping_list::{ShoppingItem, ShoppingListGroup};
use crate::repo::shopping_list;

/// Recetas planificadas en los últimos `days` días, más recientes primero.
///
/// Se toma la fecha de planificación más reciente por receta dentro de la
/// ventana; el resultado sirve al agente para evitar repeticiones. La ventana se
/// calcula en hora **local** (`localtime`) para ser coherente con el resto del
/// sistema (`current_week`, tests y creación de entradas).
pub async fn recent_recipes(pool: &SqlitePool, days: u32) -> AppResult<Vec<RecentRecipe>> {
    let modifier = format!("-{days} days");
    let rows: Vec<RecentRecipe> = sqlx::query_as(
        "SELECT r.id, r.name, r.category, MAX(CAST(e.date AS TEXT)) AS last_planned \
         FROM meal_plan_entries e JOIN recipes r ON r.id = e.recipe_id \
         WHERE e.date >= date('now', 'localtime', ?) \
         GROUP BY r.id, r.name, r.category \
         ORDER BY last_planned DESC, r.name ASC",
    )
    .bind(modifier)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Fija el plan semanal de forma transaccional y reconstruye la compra.
///
/// Valida que todos los `recipe_id` existan **antes** de escribir. Normaliza
/// cada fecha a `YYYY-MM-DD` y, para cada fecha distinta presente en `entries`,
/// elimina las entradas existentes y luego inserta las nuevas. Tras escribir,
/// regenera los ítems derivados del plan conservando los ítems manuales.
///
/// **Semántica de reconstrucción**: la lista de la compra es una **única lista
/// semanal global**; `shopping_list::replace_consolidated` borra *todos* los
/// ítems con `source_plan_id` no nulo (no solo los del rango) y los regenera a
/// partir de las entradas del plan leídas entre `start` y `end`. Devuelve el
/// número de entradas creadas y la lista agrupada resultante.
pub async fn set_weekly_plan(
    pool: &SqlitePool,
    entries: &[StructuredPlanEntry],
) -> AppResult<(i64, Vec<ShoppingListGroup>)> {
    if entries.is_empty() {
        return Ok((0, shopping_list::list_grouped(pool).await?));
    }

    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    // Validación previa: todos los `recipe_id` deben existir.
    let mut recipe_ids: Vec<&str> = entries
        .iter()
        .map(|entry| entry.recipe_id.as_str())
        .collect();
    recipe_ids.sort_unstable();
    recipe_ids.dedup();
    for recipe_id in &recipe_ids {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
            .bind(recipe_id)
            .fetch_optional(&mut *tx)
            .await?;
        if exists.is_none() {
            return Err(AppError::Validation(format!(
                "receta no encontrada: {recipe_id}"
            )));
        }
    }

    // Normaliza cada fecha a `YYYY-MM-DD`, alineada con `entries`.
    let canonical_dates: Vec<String> = entries
        .iter()
        .map(|entry| {
            canonical_iso_date(&entry.date)
                .ok_or_else(|| AppError::Validation(format!("fecha inválida: {}", entry.date)))
        })
        .collect::<AppResult<Vec<_>>>()?;

    // Fechas distintas cubiertas por el plan.
    let mut dates: Vec<&str> = canonical_dates.iter().map(String::as_str).collect();
    dates.sort_unstable();
    dates.dedup();

    for date in &dates {
        sqlx::query("DELETE FROM meal_plan_entries WHERE date = ?")
            .bind(date)
            .execute(&mut *tx)
            .await?;
    }

    for (entry, date) in entries.iter().zip(&canonical_dates) {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(date)
        .bind(&entry.meal_type)
        .bind(&entry.recipe_id)
        .bind(&entry.role)
        .bind(entry.servings)
        .execute(&mut *tx)
        .await?;
    }

    // Reconstruye la compra en el rango cubierto, dentro de la misma transacción.
    let start = dates.first().copied().unwrap_or_default();
    let end = dates.last().copied().unwrap_or_default();
    let drafts = shopping_list::collect_drafts(&mut tx, start, end).await?;
    shopping_list::replace_consolidated(&mut tx, &drafts).await?;

    tx.commit().await?;

    let groups = shopping_list::list_grouped(pool).await?;
    Ok((entries.len() as i64, groups))
}

/// Añade ingredientes a la lista: suma a un ítem manual existente o inserta uno.
///
/// La coincidencia se hace por clave normalizada del nombre y unidad canónica.
/// Al sumar, la cantidad previa se reconvierte a su forma canónica para evitar
/// mezclar escalas (p. ej. litros con mililitros). Toda la operación es atómica.
pub async fn add_shopping_items(
    pool: &SqlitePool,
    items: &[IngredientInput],
) -> AppResult<Vec<ShoppingListGroup>> {
    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    // Ítems manuales existentes, indexados por (clave normalizada, unidad canónica).
    let existing: Vec<ShoppingItem> = sqlx::query_as(
        "SELECT id, name, amount, unit, category, checked, \
         CAST(updated_at AS TEXT) AS updated_at \
         FROM shopping_list_items WHERE source_plan_id IS NULL",
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut index: HashMap<(String, String), (String, f64)> = HashMap::new();
    for item in &existing {
        let canonical = canonicalize(item.amount, &item.unit);
        // La clave de conciliación normaliza también la unidad para que, en la
        // familia `Unknown`, `"Cajas"` y `"cajas"` se traten como la misma.
        index.insert(
            (
                normalize_name(&item.name),
                normalize_unit(&canonical.display_unit),
            ),
            (item.id.clone(), canonical.amount),
        );
    }

    for input in items {
        let normalized = normalize_name(&input.name);
        let canonical = canonicalize(input.amount, &input.unit);
        let key = (normalized.clone(), normalize_unit(&canonical.display_unit));

        if let Some((id, current)) = index.get(&key) {
            let total = round2(current + canonical.amount);
            sqlx::query(
                "UPDATE shopping_list_items \
                 SET amount = ?, unit = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(total)
            .bind(&canonical.display_unit)
            .bind(id)
            .execute(&mut *tx)
            .await?;
            index.insert(key, (id.clone(), total));
        } else {
            let category = match input
                .category
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                Some(category) => category.to_string(),
                None => categorize(&normalized).to_string(),
            };
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO shopping_list_items \
                 (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
                 VALUES (?, ?, ?, ?, ?, 0, NULL, CURRENT_TIMESTAMP)",
            )
            .bind(&id)
            .bind(&input.name)
            .bind(round2(canonical.amount))
            .bind(&canonical.display_unit)
            .bind(&category)
            .execute(&mut *tx)
            .await?;
            index.insert(key, (id, round2(canonical.amount)));
        }
    }

    tx.commit().await?;

    shopping_list::list_grouped(pool).await
}

/// Elimina los ítems marcados como comprados y devuelve cuántos se borraron.
pub async fn delete_completed(pool: &SqlitePool) -> AppResult<u64> {
    let result = sqlx::query("DELETE FROM shopping_list_items WHERE checked = 1")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
