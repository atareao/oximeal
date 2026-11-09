//! Tests de integración de la reconciliación de arranque de la lista de la compra.
//!
//! Fusiona las líneas **manuales** heredadas (`source_plan_id IS NULL AND
//! source_recipe_id IS NULL`) que comparten clave de fusión: nombre normalizado +
//! familia de unidad + unidad de presentación canónica. Los tests siembran filas
//! con SQL directo para controlar ids y orígenes.

mod common;

use oximeal_backend::domain::units::canonicalize;
use oximeal_backend::state::AppState;

/// Siembra una fila en `shopping_list_items` con id y orígenes controlados.
#[allow(clippy::too_many_arguments)]
async fn seed_item(
    state: &AppState,
    id: &str,
    name: &str,
    amount: f64,
    unit: &str,
    category: &str,
    checked: i64,
    source_plan_id: Option<&str>,
    source_recipe_id: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO shopping_list_items \
         (id, name, amount, unit, category, checked, source_plan_id, source_recipe_id, client_id, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind(id)
    .bind(name)
    .bind(amount)
    .bind(unit)
    .bind(category)
    .bind(checked)
    .bind(source_plan_id)
    .bind(source_recipe_id)
    .bind(Option::<String>::None)
    .execute(&state.pool)
    .await
    .expect("sembrar ítem");
}

/// Cuenta las filas con un nombre dado.
async fn count_by_name(state: &AppState, name: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE name = ?")
        .bind(name)
        .fetch_one(&state.pool)
        .await
        .expect("contar ítems por nombre")
}

/// Lee `(id, amount, unit, checked)` de todas las filas con un nombre dado.
async fn rows_by_name(state: &AppState, name: &str) -> Vec<(String, f64, String, i64)> {
    sqlx::query_as::<_, (String, f64, String, i64)>(
        "SELECT id, amount, unit, checked FROM shopping_list_items WHERE name = ?",
    )
    .bind(name)
    .fetch_all(&state.pool)
    .await
    .expect("leer filas por nombre")
}

/// Ejecuta la reconciliación y propaga el error si falla.
async fn reconcile(state: &AppState) {
    oximeal_backend::services::shopping_reconcile::reconcile(&state.pool)
        .await
        .expect("reconciliar la lista");
}

/// Dos líneas manuales «Tomate» de 2 ud y 3 ud se fusionan en una de 5 ud.
#[tokio::test]
async fn reconcile_merges_duplicate_manual_lines() {
    let state = common::test_state().await;
    seed_item(
        &state,
        "tm-a",
        "Tomate",
        2.0,
        "ud",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;
    seed_item(
        &state,
        "tm-b",
        "Tomate",
        3.0,
        "ud",
        "Verdulería",
        1,
        None,
        None,
    )
    .await;

    reconcile(&state).await;

    assert_eq!(
        count_by_name(&state, "Tomate").await,
        1,
        "debe quedar una línea"
    );
    let rows = rows_by_name(&state, "Tomate").await;
    assert_eq!(rows[0].0, "tm-a", "el supervivente es el id menor");
    assert!((rows[0].1 - 5.0).abs() < 1e-9, "2 ud + 3 ud = 5 ud");
    assert_eq!(rows[0].2, "ud");
    assert_eq!(rows[0].3, 1, "checked = 1 si alguna lo estaba");
}

/// «Harina» 1 kg + 500 g se canonicalizan y fusionan en una línea de 1500 g.
#[tokio::test]
async fn reconcile_canonicalizes_compatible_units() {
    let state = common::test_state().await;
    seed_item(
        &state, "hr-a", "Harina", 1.0, "kg", "Despensa", 0, None, None,
    )
    .await;
    seed_item(
        &state, "hr-b", "Harina", 500.0, "g", "Despensa", 0, None, None,
    )
    .await;

    reconcile(&state).await;

    assert_eq!(
        count_by_name(&state, "Harina").await,
        1,
        "debe quedar una línea"
    );
    let rows = rows_by_name(&state, "Harina").await;
    assert_eq!(rows[0].0, "hr-a", "el supervivente es el id menor");
    let canonical = canonicalize(rows[0].1, &rows[0].2);
    assert!(
        (canonical.amount - 1500.0).abs() < 1e-9,
        "1 kg + 500 g debe equivaler a 1500 g, pero fue {} {}",
        rows[0].1,
        rows[0].2
    );
}

/// Unidades de familias distintas («ud» y «g») no se fusionan.
#[tokio::test]
async fn reconcile_keeps_incompatible_units_separate() {
    let state = common::test_state().await;
    seed_item(
        &state,
        "tj-a",
        "Tomate",
        2.0,
        "ud",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;
    seed_item(
        &state,
        "tj-b",
        "Tomate",
        500.0,
        "g",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;

    reconcile(&state).await;

    assert_eq!(
        count_by_name(&state, "Tomate").await,
        2,
        "familias distintas no se fusionan"
    );
}

/// Una línea manual y una del plan no se fusionan aunque compartan clave.
#[tokio::test]
async fn reconcile_does_not_merge_across_sources() {
    let state = common::test_state().await;
    seed_item(
        &state,
        "tp-manual",
        "Tomate",
        2.0,
        "ud",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;
    seed_item(
        &state,
        "tp-plan",
        "Tomate",
        4.0,
        "ud",
        "Verdulería",
        0,
        Some("consolidated"),
        None,
    )
    .await;

    reconcile(&state).await;

    assert_eq!(
        count_by_name(&state, "Tomate").await,
        2,
        "orígenes distintos no se fusionan"
    );
}

/// El supervivente es el id lexicográficamente menor y hereda el `checked`.
#[tokio::test]
async fn reconcile_picks_lowest_id_as_survivor_and_or_checks() {
    let state = common::test_state().await;
    seed_item(
        &state,
        "zz",
        "Tomate",
        1.0,
        "ud",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;
    seed_item(
        &state,
        "aa",
        "Tomate",
        4.0,
        "ud",
        "Verdulería",
        1,
        None,
        None,
    )
    .await;

    reconcile(&state).await;

    let rows = rows_by_name(&state, "Tomate").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "aa", "«aa» < «zz» lexicográficamente");
    assert!((rows[0].1 - 5.0).abs() < 1e-9, "1 ud + 4 ud = 5 ud");
    assert_eq!(rows[0].3, 1, "checked = 1 si alguna lo estaba");
}

/// Reconciliar dos veces no cambia nada y escribe la marca persistente.
#[tokio::test]
async fn reconcile_is_idempotent_and_writes_marker() {
    let state = common::test_state().await;
    seed_item(
        &state,
        "id-a",
        "Tomate",
        2.0,
        "ud",
        "Verdulería",
        0,
        None,
        None,
    )
    .await;
    seed_item(
        &state,
        "id-b",
        "Tomate",
        3.0,
        "ud",
        "Verdulería",
        1,
        None,
        None,
    )
    .await;

    reconcile(&state).await;
    let snapshot: String = format!(
        "{:?}",
        sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT id, amount, checked FROM shopping_list_items ORDER BY id"
        )
        .fetch_all(&state.pool)
        .await
        .expect("leer snapshot")
    );

    reconcile(&state).await;
    let again: String = format!(
        "{:?}",
        sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT id, amount, checked FROM shopping_list_items ORDER BY id"
        )
        .fetch_all(&state.pool)
        .await
        .expect("leer snapshot de nuevo")
    );

    assert_eq!(snapshot, again, "la segunda ejecución no debe cambiar nada");
    let marked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_backfills WHERE name = 'merge-duplicate-lines'",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar marcas");
    assert_eq!(
        marked, 1,
        "la marca persistente debe escribirse una sola vez"
    );
}

/// Unidades desconocidas se fusionan insensibles a mayúsculas («cajas»/«Cajas»).
#[tokio::test]
async fn reconcile_merges_unknown_units_case_insensitively() {
    let state = common::test_state().await;
    seed_item(
        &state, "cj-a", "Huevos", 2.0, "cajas", "Otros", 0, None, None,
    )
    .await;
    seed_item(
        &state, "cj-b", "Huevos", 3.0, "Cajas", "Otros", 0, None, None,
    )
    .await;

    reconcile(&state).await;

    let rows = rows_by_name(&state, "Huevos").await;
    assert_eq!(rows.len(), 1, "la unidad desconocida normalizada coincide");
    assert_eq!(rows[0].0, "cj-a", "el supervivente es el id menor");
    assert!((rows[0].1 - 5.0).abs() < 1e-9, "2 cajas + 3 cajas = 5");
}

/// Una única línea manual no se toca.
#[tokio::test]
async fn reconcile_leaves_single_lines_untouched() {
    let state = common::test_state().await;
    seed_item(&state, "solo", "Leche", 1.0, "l", "Lácteos", 1, None, None).await;

    reconcile(&state).await;

    let rows = rows_by_name(&state, "Leche").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "solo");
    assert!((rows[0].1 - 1.0).abs() < 1e-9, "amount intacto");
    assert_eq!(rows[0].2, "l", "unit intacta");
    assert_eq!(rows[0].3, 1, "checked intacto");
}
