//! Reconciliación de un solo uso de la lista de la compra.
//!
//! Tras #129 el volcado del plan es declarativo y los orígenes atribuidos
//! (`source_plan_id` / `source_recipe_id`) ya deduplican dentro de sí mismos. El
//! único resto del antiguo `from-plan` **aditivo** son las **líneas manuales
//! heredadas** (`source_plan_id IS NULL AND source_recipe_id IS NULL`), que se
//! creaban por duplicado al repetir el volcado con la misma clave.
//!
//! Este servicio fusiona esas líneas manuales duplicadas **una sola vez**,
//! guardado por un marcador persistente en `shopping_backfills`: si el marcador
//! ya existe, no hace nada. La clave de fusión es
//! `(nombre normalizado, familia de unidad, unidad de presentación canónica)`,
//! coherente con `domain::consolidate` y con `agent::add_shopping_items`. La
//! reconciliación no se puede expresar en SQL porque depende de la
//! canonicalización de unidades (familia + `display_unit`).

use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::domain::consolidate::round2;
use crate::domain::normalize::normalize_name;
use crate::domain::units::{UnitFamily, canonicalize};
use crate::error::AppResult;

/// Marcador del backfill que fusiona las líneas manuales duplicadas.
pub const MARK_MERGE_DUPLICATE_LINES: &str = "merge-duplicate-lines";

/// Clave de fusión: nombre normalizado + familia de unidad + unidad canónica.
type MergeKey = (String, UnitFamily, String);

/// Fila manual leída de `shopping_list_items` para la reconciliación.
struct Row {
    id: String,
    amount: f64,
    unit: String,
    checked: i64,
}

/// Fusiona las líneas manuales duplicadas heredadas del antiguo `from-plan`
/// aditivo, una sola vez por base de datos.
///
/// Si el marcador [`MARK_MERGE_DUPLICATE_LINES`] ya está presente, devuelve
/// `Ok(())` sin tocar la base. En caso contrario, dentro de una única
/// transacción `BEGIN IMMEDIATE`:
///
/// 1. Lee las líneas **manuales** (`source_plan_id` y `source_recipe_id` nulos),
///    ordenadas por `id` ascendente.
/// 2. Las agrupa por [`MergeKey`].
/// 3. Por cada grupo con más de una fila: la de `id` menor sobrevive (conserva
///    su `name` y su `category`), se le fija la cantidad total canonicalizada
///    expresada en su unidad, se hereda `checked = 1` si alguna fila lo estaba y
///    se eliminan las demás.
/// 4. Escribe el marcador y confirma.
///
/// Las familias no aditivas (`ToTaste`) resultan en `0.0`. Los grupos se iteran
/// en orden determinista (por el `id` del supervivente) para que los updates
/// sean reproducibles. Los orígenes distintos (manual / receta / plan) **nunca**
/// se fusionan entre sí.
pub async fn reconcile(pool: &SqlitePool) -> AppResult<()> {
    let already_applied: Option<String> =
        sqlx::query_scalar("SELECT name FROM shopping_backfills WHERE name = ?")
            .bind(MARK_MERGE_DUPLICATE_LINES)
            .fetch_optional(pool)
            .await?;
    if already_applied.is_some() {
        return Ok(());
    }

    // Escritura leer-luego-escribir en WAL: BEGIN IMMEDIATE evita
    // SQLITE_BUSY_SNAPSHOT (mismo patrón que `repo::shopping_list`).
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let rows: Vec<(String, String, f64, String, i64)> = sqlx::query_as(
        "SELECT id, name, amount, unit, checked FROM shopping_list_items \
         WHERE source_plan_id IS NULL AND source_recipe_id IS NULL \
         ORDER BY id ASC",
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut groups: HashMap<MergeKey, Vec<Row>> = HashMap::new();
    for (id, name, amount, unit, checked) in rows {
        let key = merge_key(&name, amount, &unit);
        groups.entry(key).or_default().push(Row {
            id,
            amount,
            unit,
            checked,
        });
    }

    // Orden determinista: por el `id` del supervivente (la primera fila, ya que
    // el `SELECT` viene ordenado por `id` ascendente).
    let mut mergeable: Vec<&Vec<Row>> = groups.values().filter(|rows| rows.len() > 1).collect();
    mergeable.sort_by(|a, b| a[0].id.cmp(&b[0].id));

    for group in mergeable {
        let survivor = &group[0];
        let canonical = canonicalize(survivor.amount, &survivor.unit);

        let total_base: f64 = round2(
            group
                .iter()
                .map(|row| canonicalize(row.amount, &row.unit).amount)
                .sum(),
        );

        // La cantidad total se expresa en la unidad de presentación del
        // supervivente. `canonicalize(1.0, unit)` es la equivalencia de una
        // unidad de presentación en unidades base.
        let factor = canonicalize(1.0, &survivor.unit).amount;
        let amount = if canonical.family == UnitFamily::ToTaste || factor <= 0.0 {
            0.0
        } else {
            round2(total_base / factor)
        };

        let checked = i64::from(group.iter().any(|row| row.checked != 0));

        sqlx::query("UPDATE shopping_list_items SET amount = ?, checked = ? WHERE id = ?")
            .bind(amount)
            .bind(checked)
            .bind(&survivor.id)
            .execute(&mut *tx)
            .await?;

        for row in &group[1..] {
            sqlx::query("DELETE FROM shopping_list_items WHERE id = ?")
                .bind(&row.id)
                .execute(&mut *tx)
                .await?;
        }
    }

    sqlx::query("INSERT OR IGNORE INTO shopping_backfills (name) VALUES (?)")
        .bind(MARK_MERGE_DUPLICATE_LINES)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

/// Construye la clave de fusión de una fila manual.
fn merge_key(name: &str, amount: f64, unit: &str) -> MergeKey {
    let canonical = canonicalize(amount, unit);
    (
        normalize_name(name),
        canonical.family,
        canonical.display_unit,
    )
}
