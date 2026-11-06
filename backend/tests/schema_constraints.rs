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

/// Migración `0007`: crea `tags`, `recipe_tags`, el índice y la unicidad de `slug`.
#[tokio::test]
async fn migration_0007_creates_tags_schema() {
    let state = common::test_state().await;

    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .expect("sqlite_master: tablas");
    assert!(tables.iter().any(|t| t == "tags"), "falta la tabla tags");
    assert!(
        tables.iter().any(|t| t == "recipe_tags"),
        "falta la tabla recipe_tags"
    );

    let indexes: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'index'")
            .fetch_all(&state.pool)
            .await
            .expect("sqlite_master: índices");
    assert!(
        indexes.iter().any(|i| i == "idx_recipe_tags_tag_id"),
        "falta idx_recipe_tags_tag_id: {indexes:?}"
    );

    // `tags.slug` tiene una restricción UNIQUE: un slug duplicado debe ser
    // rechazado. Se usa un slug libre (el catálogo automático de `0009` ya ocupa
    // los canónicos).
    sqlx::query("INSERT INTO tags (id, name, slug) VALUES ('t1', 'Libre', 'libre')")
        .execute(&state.pool)
        .await
        .expect("insert tag");
    let duplicate = sqlx::query("INSERT INTO tags (id, name, slug) VALUES ('t2', 'Otro', 'libre')")
        .execute(&state.pool)
        .await;
    assert!(
        duplicate.is_err(),
        "tags.slug debe ser UNIQUE (rechaza duplicados)"
    );
}

/// `recipe_tags` declara FK `ON DELETE CASCADE` a `recipes(id)` y `tags(id)`.
#[tokio::test]
async fn recipe_tags_has_cascade_foreign_keys() {
    let state = common::test_state().await;

    let rows = sqlx::query("PRAGMA foreign_key_list('recipe_tags')")
        .fetch_all(&state.pool)
        .await
        .expect("foreign_key_list recipe_tags");
    assert!(!rows.is_empty(), "recipe_tags debe tener claves foráneas");

    let on_delete = |table: &str| {
        rows.iter().find_map(|row| {
            let referenced: String = row.get("table");
            (referenced == table).then(|| row.get::<String, _>("on_delete"))
        })
    };
    assert_eq!(
        on_delete("recipes").as_deref(),
        Some("CASCADE"),
        "FK recipe_tags.recipe_id debe ser ON DELETE CASCADE"
    );
    assert_eq!(
        on_delete("tags").as_deref(),
        Some("CASCADE"),
        "FK recipe_tags.tag_id debe ser ON DELETE CASCADE"
    );
}

/// La migración `0007` es aditiva: no toca `recipes` ni `recipes_fts`, y
/// revertirla (DROP de sus tablas) no pierde recetas.
#[tokio::test]
async fn migration_0007_is_additive_and_reversible() {
    let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
        .await
        .expect("conectar memoria");

    for sql in [
        include_str!("../migrations/0001_init.sql"),
        include_str!("../migrations/0002_fts_triggers.sql"),
        include_str!("../migrations/0003_fts_diacritics_and_indexes.sql"),
        include_str!("../migrations/0004_sessions.sql"),
        include_str!("../migrations/0005_schema_constraints.sql"),
        include_str!("../migrations/0006_shopping_item_client_id.sql"),
    ] {
        sqlx::raw_sql(sql)
            .execute(&mut conn)
            .await
            .expect("migración previa");
    }

    sqlx::query("INSERT INTO recipes (id, name, category, ingredients, servings) VALUES ('r1', 'CafeteriaTest', 'main', '[]', 2)")
        .execute(&mut conn)
        .await
        .expect("insert receta");

    // Se lee en runtime (no `include_str!`) para que este test compile aunque la
    // migración 0007 todavía no exista: así el RED falla por aserción, no por
    // error de compilación.
    let migration = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations/0007_recipe_tags.sql"),
    )
    .expect("leer migración 0007");

    sqlx::raw_sql(sqlx::AssertSqlSafe(migration))
        .execute(&mut conn)
        .await
        .expect("migración 0007");

    let recipes: i64 = sqlx::query_scalar("SELECT count(*) FROM recipes")
        .fetch_one(&mut conn)
        .await
        .expect("contar recetas");
    assert_eq!(recipes, 1, "0007 no debe modificar filas de recipes");

    let hits: i64 =
        sqlx::query_scalar("SELECT count(*) FROM recipes_fts WHERE recipes_fts MATCH 'Cafe'")
            .fetch_one(&mut conn)
            .await
            .expect("fts");
    assert_eq!(hits, 1, "0007 no debe tocar recipes_fts");

    // Revertir 0007 no pierde recetas.
    sqlx::raw_sql("DROP TABLE recipe_tags; DROP TABLE tags;")
        .execute(&mut conn)
        .await
        .expect("revertir 0007");

    let recipes: i64 = sqlx::query_scalar("SELECT count(*) FROM recipes")
        .fetch_one(&mut conn)
        .await
        .expect("contar recetas tras revertir");
    assert_eq!(recipes, 1, "revertir 0007 no pierde recetas");
}

/// Migración `0008`: columna `source_recipe_id`, su índice y la tabla
/// `shopping_recipe_sources` (PK, FK en cascada y CHECK de raciones).
#[tokio::test]
async fn migration_0008_adds_recipe_source_column_index_and_table() {
    let state = common::test_state().await;

    let columns = sqlx::query("PRAGMA table_info('shopping_list_items')")
        .fetch_all(&state.pool)
        .await
        .expect("table_info shopping_list_items");
    assert!(
        columns
            .iter()
            .any(|row| row.get::<String, _>("name") == "source_recipe_id"),
        "falta la columna shopping_list_items.source_recipe_id"
    );

    let indexes: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'index'")
            .fetch_all(&state.pool)
            .await
            .expect("sqlite_master: índices");
    assert!(
        indexes
            .iter()
            .any(|name| name == "idx_shopping_source_recipe_id"),
        "falta idx_shopping_source_recipe_id: {indexes:?}"
    );

    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(&state.pool)
            .await
            .expect("sqlite_master: tablas");
    assert!(
        tables.iter().any(|name| name == "shopping_recipe_sources"),
        "falta la tabla shopping_recipe_sources: {tables:?}"
    );

    let pk: Vec<String> = sqlx::query("PRAGMA table_info('shopping_recipe_sources')")
        .fetch_all(&state.pool)
        .await
        .expect("table_info shopping_recipe_sources")
        .iter()
        .filter(|row| row.get::<i64, _>("pk") > 0)
        .map(|row| row.get::<String, _>("name"))
        .collect();
    assert_eq!(pk, vec!["recipe_id".to_string()], "recipe_id debe ser PK");

    let rows = sqlx::query("PRAGMA foreign_key_list('shopping_recipe_sources')")
        .fetch_all(&state.pool)
        .await
        .expect("foreign_key_list shopping_recipe_sources");
    let on_delete = rows.iter().find_map(|row| {
        let referenced: String = row.get("table");
        (referenced == "recipes").then(|| row.get::<String, _>("on_delete"))
    });
    assert_eq!(
        on_delete.as_deref(),
        Some("CASCADE"),
        "FK shopping_recipe_sources.recipe_id debe ser ON DELETE CASCADE"
    );
}

/// `shopping_recipe_sources` aplica el CHECK `servings >= 1` y desregistra la
/// fuente en cascada al borrar la receta.
#[tokio::test]
async fn shopping_recipe_sources_enforces_servings_check_and_cascade() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rs', 'Fuente', 'main', '[]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("insert receta");

    let below_one =
        sqlx::query("INSERT INTO shopping_recipe_sources (recipe_id, servings) VALUES ('rs', 0)")
            .execute(&state.pool)
            .await;
    assert!(
        below_one.is_err(),
        "servings < 1 debe ser rechazado por el CHECK"
    );

    sqlx::query("INSERT INTO shopping_recipe_sources (recipe_id, servings) VALUES ('rs', 2)")
        .execute(&state.pool)
        .await
        .expect("insert fuente válida");

    // Borrar la receta desregistra su fuente (cascade).
    sqlx::query("DELETE FROM recipes WHERE id = 'rs'")
        .execute(&state.pool)
        .await
        .expect("borrar receta");
    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_recipe_sources WHERE recipe_id = 'rs'")
            .fetch_one(&state.pool)
            .await
            .expect("contar fuentes");
    assert_eq!(remaining, 0, "la fuente se elimina en cascada");
}
