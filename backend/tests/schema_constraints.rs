//! Tests de integridad del esquema: FK ON DELETE y CHECK de dominio (issue #4).

mod common;

use sqlx::Connection;
use sqlx::Row;

#[tokio::test]
async fn fk_recipe_on_delete_is_restrict() {
    let state = common::test_state().await;
    let rows = sqlx::query("PRAGMA foreign_key_list('meal_plan_entries')")
        .fetch_all(&state.pool)
        .await
        .expect("foreign_key_list");
    let on_delete = rows.iter().find_map(|r| {
        let table: String = r.get("table");
        (table == "recipes").then(|| r.get::<String, _>("on_delete"))
    });
    assert_eq!(
        on_delete.as_deref(),
        Some("RESTRICT"),
        "la FK meal_plan_entries.recipe_id debe declarar ON DELETE RESTRICT"
    );
}

#[tokio::test]
async fn delete_referenced_recipe_is_rejected() {
    let state = common::test_state().await;

    // La FK declarada debe ser ON DELETE RESTRICT.
    let rows = sqlx::query("PRAGMA foreign_key_list('meal_plan_entries')")
        .fetch_all(&state.pool)
        .await
        .expect("foreign_key_list");
    let on_delete = rows.iter().find_map(|r| {
        let table: String = r.get("table");
        (table == "recipes").then(|| r.get::<String, _>("on_delete"))
    });
    assert_eq!(
        on_delete.as_deref(),
        Some("RESTRICT"),
        "la FK meal_plan_entries.recipe_id debe declarar ON DELETE RESTRICT"
    );

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rref', 'Referenciada', 'main', '[]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("insert receta");
    sqlx::query(
        "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
         VALUES ('eref', '2026-02-03', 'lunch', 'rref', 'main', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("insert entrada");

    let result = sqlx::query("DELETE FROM recipes WHERE id = 'rref'")
        .execute(&state.pool)
        .await;
    assert!(
        result.is_err(),
        "no debe permitirse borrar una receta referenciada por el plan"
    );

    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM meal_plan_entries WHERE id = 'eref'")
            .fetch_one(&state.pool)
            .await
            .expect("count entrada");
    assert_eq!(remaining, 1, "la entrada del plan debe seguir existiendo");
}

#[tokio::test]
async fn check_meal_plan_servings_rejects_below_one() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rp0', 'Plan', 'main', '[]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("insert receta válida");

    let result = sqlx::query(
        "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
         VALUES ('ep0', '2026-02-04', 'lunch', 'rp0', 'main', 0)",
    )
    .execute(&state.pool)
    .await;
    assert!(
        result.is_err(),
        "servings < 1 en meal_plan_entries debe ser rechazado por el CHECK"
    );
}

#[tokio::test]
async fn check_amount_rejects_negative() {
    let state = common::test_state().await;
    let result = sqlx::query(
        "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked) \
         VALUES ('a', 'x', -1.0, 'ud', 'Otros', 0)",
    )
    .execute(&state.pool)
    .await;
    assert!(
        result.is_err(),
        "amount negativo debe ser rechazado por el CHECK"
    );
}

#[tokio::test]
async fn check_amount_accepts_zero() {
    let state = common::test_state().await;
    sqlx::query(
        "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked) \
         VALUES ('b', 'x', 0.0, 'ud', 'Otros', 0)",
    )
    .execute(&state.pool)
    .await
    .expect("amount = 0 debe ser aceptado");
}

#[tokio::test]
async fn check_recipes_servings_rejects_below_one() {
    let state = common::test_state().await;
    let result = sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('r0', 'Test', 'main', '[]', 0)",
    )
    .execute(&state.pool)
    .await;
    assert!(
        result.is_err(),
        "servings < 1 debe ser rechazado por el CHECK"
    );
}

#[tokio::test]
async fn check_prep_time_rejects_negative() {
    let state = common::test_state().await;
    let result = sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, prep_time_minutes) \
         VALUES ('r1', 'Test', 'main', '[]', -1)",
    )
    .execute(&state.pool)
    .await;
    assert!(
        result.is_err(),
        "prep_time_minutes negativo debe ser rechazado por el CHECK"
    );
}

#[tokio::test]
async fn check_servings_accepts_null() {
    let state = common::test_state().await;
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('r2', 'Test', 'main', '[]', NULL)",
    )
    .execute(&state.pool)
    .await
    .expect("servings NULL debe ser aceptado");
}

#[tokio::test]
async fn migration_preserves_data_and_fts() {
    let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
        .await
        .expect("conectar memoria");

    for sql in [
        include_str!("../migrations/0001_init.sql"),
        include_str!("../migrations/0002_fts_triggers.sql"),
        include_str!("../migrations/0003_fts_diacritics_and_indexes.sql"),
        include_str!("../migrations/0004_sessions.sql"),
    ] {
        sqlx::raw_sql(sql)
            .execute(&mut conn)
            .await
            .expect("migración previa");
    }

    sqlx::query("INSERT INTO recipes (id, name, category, ingredients, servings) VALUES ('r1', 'CafeteriaTest', 'main', '[]', 2)")
        .execute(&mut conn).await.expect("insert receta");
    sqlx::query("INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) VALUES ('e1', '2026-01-02', 'lunch', 'r1', 'main', 2)")
        .execute(&mut conn).await.expect("insert entrada");
    sqlx::query("INSERT INTO shopping_list_items (id, name, amount, unit, category, checked) VALUES ('s1', 'Tomate', 2.0, 'ud', 'Otros', 0)")
        .execute(&mut conn).await.expect("insert item");

    sqlx::raw_sql(include_str!("../migrations/0005_schema_constraints.sql"))
        .execute(&mut conn)
        .await
        .expect("migración 0005");

    let recipes: i64 = sqlx::query_scalar("SELECT count(*) FROM recipes")
        .fetch_one(&mut conn)
        .await
        .expect("c1");
    assert_eq!(recipes, 1, "los datos de recipes deben preservarse");
    let entries: i64 = sqlx::query_scalar("SELECT count(*) FROM meal_plan_entries")
        .fetch_one(&mut conn)
        .await
        .expect("c2");
    assert_eq!(
        entries, 1,
        "los datos de meal_plan_entries deben preservarse"
    );
    let items: i64 = sqlx::query_scalar("SELECT count(*) FROM shopping_list_items")
        .fetch_one(&mut conn)
        .await
        .expect("c3");
    assert_eq!(
        items, 1,
        "los datos de shopping_list_items deben preservarse"
    );

    let hits: i64 =
        sqlx::query_scalar("SELECT count(*) FROM recipes_fts WHERE recipes_fts MATCH 'Cafe'")
            .fetch_one(&mut conn)
            .await
            .expect("fts");
    assert_eq!(hits, 1, "el índice FTS5 debe seguir encontrando la receta");

    let rows = sqlx::query("PRAGMA foreign_key_list('meal_plan_entries')")
        .fetch_all(&mut conn)
        .await
        .expect("fk list");
    let on_delete = rows.iter().find_map(|r| {
        let table: String = r.get("table");
        (table == "recipes").then(|| r.get::<String, _>("on_delete"))
    });
    assert_eq!(
        on_delete.as_deref(),
        Some("RESTRICT"),
        "FK RESTRICT tras migrar"
    );

    let err = sqlx::query("INSERT INTO shopping_list_items (id, name, amount, unit, category, checked) VALUES ('s2','X',-1.0,'ud','Otros',0)")
        .execute(&mut conn).await;
    assert!(
        err.is_err(),
        "el CHECK de amount debe estar activo tras migrar"
    );
}
