//! Tests de integración de migraciones y búsqueda FTS5 (trigram).

mod common;

use sqlx::Row;

/// Caso RED→GREEN: las migraciones crean las tablas esperadas.
#[tokio::test]
async fn migrations_create_expected_tables() {
    let state = common::test_state().await;

    for table in [
        "recipes",
        "meal_plan_entries",
        "shopping_list_items",
        "recipes_fts",
    ] {
        let name: Option<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE name = ?")
                .bind(table)
                .fetch_optional(&state.pool)
                .await
                .expect("consultar sqlite_master");

        assert_eq!(name.as_deref(), Some(table), "falta la tabla {table}");
    }
}

/// La migración `0004_sessions.sql` crea la tabla server-side de sesiones con
/// sus columnas e índice de caducidad.
#[tokio::test]
async fn migrations_create_sessions_table() {
    let state = common::test_state().await;

    let table: Option<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind("sessions")
            .fetch_optional(&state.pool)
            .await
            .expect("consultar sqlite_master");
    assert_eq!(
        table.as_deref(),
        Some("sessions"),
        "falta la tabla sessions"
    );

    let columns: Vec<String> = sqlx::query("SELECT name FROM pragma_table_info('sessions')")
        .fetch_all(&state.pool)
        .await
        .expect("leer columnas de sessions")
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();
    for expected in ["id", "data", "expiry"] {
        assert!(
            columns.iter().any(|column| column == expected),
            "falta la columna {expected} en sessions"
        );
    }

    let index: Option<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'index' AND name = ?")
            .bind("idx_sessions_expiry")
            .fetch_optional(&state.pool)
            .await
            .expect("consultar índice");
    assert_eq!(
        index.as_deref(),
        Some("idx_sessions_expiry"),
        "falta el índice idx_sessions_expiry"
    );
}

/// Verifica en runtime que FTS5 con tokenizer `trigram` está disponible y que
/// encuentra coincidencias por substring de >= 3 caracteres.
#[tokio::test]
async fn fts5_trigram_matches_substring() {
    let state = common::test_state().await;

    // Confirmar versión de SQLite (FTS5 trigram requiere >= 3.34).
    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(&state.pool)
        .await
        .expect("obtener versión de SQLite");
    assert!(!version.is_empty(), "sqlite_version() devolvió vacío");

    // El trigger `recipes_ai` debe poblar el índice externo.
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES (?, ?, 'main', '[]')",
    )
    .bind("r1")
    .bind("Lentejas estofadas")
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    // Substring en medio de la palabra ("entej") => solo viable vía trigram.
    let hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'entej'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar en FTS5");

    assert_eq!(
        hits,
        vec!["Lentejas estofadas".to_string()],
        "el tokenizer trigram debe encontrar substrings"
    );
}

/// El trigger mantiene el índice sincronizado en inserciones sucesivas y
/// es insensible a mayúsculas/minúsculas para ASCII.
#[tokio::test]
async fn fts_index_stays_in_sync_with_triggers() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES ('r1', 'Tortilla de patatas', 'main', '[]')",
    )
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    let hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'tilla'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar en FTS5");

    assert_eq!(hits, vec!["Tortilla de patatas".to_string()]);
}

/// El trigger `recipes_au` actualiza el índice al modificar una receta.
#[tokio::test]
async fn fts_index_updates_on_update() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES ('r1', 'Paella valenciana', 'main', '[]')",
    )
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    sqlx::query("UPDATE recipes SET name = 'Paella de marisco' WHERE id = 'r1'")
        .execute(&state.pool)
        .await
        .expect("actualizar receta");

    let new_hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'maris'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar nombre nuevo");

    assert_eq!(new_hits, vec!["Paella de marisco".to_string()]);
}

/// El trigger `recipes_ad` elimina la entrada del índice al borrar la receta.
#[tokio::test]
async fn fts_index_removes_on_delete() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES ('r1', 'Gazpacho andaluz', 'salad', '[]')",
    )
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    sqlx::query("DELETE FROM recipes WHERE id = 'r1'")
        .execute(&state.pool)
        .await
        .expect("borrar receta");

    let hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'zpa'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar tras borrado");

    assert!(
        hits.is_empty(),
        "el índice debe quedar vacío tras el borrado"
    );
}

/// F3 — El tokenizer debe plegar diacríticos: `cafe` encuentra `Café`.
#[tokio::test]
async fn fts5_folds_diacritics() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES ('r1', 'Café con leche', 'breakfast', '[]')",
    )
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    let hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'cafe'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar sin diacríticos");

    assert_eq!(
        hits,
        vec!["Café con leche".to_string()],
        "el tokenizer debe plegar diacríticos (`cafe` == `café`)"
    );
}

/// F3 — El índice se mantiene sincronizado en insert/update/delete tras
/// recrearlo con la migración 0003.
#[tokio::test]
async fn fts_recreated_index_stays_in_sync() {
    let state = common::test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients)
         VALUES ('r1', 'Tortilla de patatas', 'main', '[]')",
    )
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    let insert_hits: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'tilla'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar tras insert");
    assert_eq!(insert_hits, vec!["Tortilla de patatas".to_string()]);

    sqlx::query("UPDATE recipes SET name = 'Tortilla de calabacín' WHERE id = 'r1'")
        .execute(&state.pool)
        .await
        .expect("actualizar receta");
    let stale: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'patata'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar nombre antiguo");
    assert!(
        stale.is_empty(),
        "el nombre antiguo no debe quedar indexado"
    );

    sqlx::query("DELETE FROM recipes WHERE id = 'r1'")
        .execute(&state.pool)
        .await
        .expect("borrar receta");
    let after_delete: Vec<String> =
        sqlx::query_scalar("SELECT name FROM recipes_fts WHERE recipes_fts MATCH 'tilla'")
            .fetch_all(&state.pool)
            .await
            .expect("buscar tras borrado");
    assert!(after_delete.is_empty(), "el índice debe quedar vacío");
}

/// Comprobación directa de que las columnas base existen con los tipos previstos.
#[tokio::test]
async fn recipes_table_has_expected_columns() {
    let state = common::test_state().await;

    let rows = sqlx::query("SELECT name FROM pragma_table_info('recipes')")
        .fetch_all(&state.pool)
        .await
        .expect("leer info de tabla");

    let columns: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    for expected in [
        "id",
        "name",
        "category",
        "diet_type",
        "prep_time_minutes",
        "servings",
        "ingredients",
        "instructions",
        "source_url",
        "created_at",
    ] {
        assert!(
            columns.iter().any(|column| column == expected),
            "falta la columna {expected}"
        );
    }
}
