//! Tests de concurrencia y bloqueos de escritura en SQLite (issue #2).

mod common;

use oximeal_backend::domain::ingredient::IngredientInput;
use oximeal_backend::models::agent::StructuredPlanEntry;
use oximeal_backend::repo::{agent, shopping_list};

#[tokio::test]
async fn deferred_begin_reproduces_busy_snapshot() {
    let (pool, _) = common::test_pool().await;
    let mut conn1 = pool.acquire().await.expect("con1");
    let mut conn2 = pool.acquire().await.expect("con2");

    sqlx::query("BEGIN")
        .execute(&mut *conn1)
        .await
        .expect("BEGIN con1");
    sqlx::query("SELECT count(*) FROM recipes")
        .execute(&mut *conn1)
        .await
        .expect("SELECT con1");

    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn2)
        .await
        .expect("BEGIN IMMEDIATE con2");
    sqlx::query(
        "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
         VALUES ('00000000-0000-0000-0000-000000000000', 'Item test', 1.0, 'ud', 'Otros', 0, 'test', CURRENT_TIMESTAMP)",
    )
    .execute(&mut *conn2)
    .await
    .expect("INSERT con2");
    sqlx::query("COMMIT")
        .execute(&mut *conn2)
        .await
        .expect("COMMIT con2");

    let result = sqlx::query(
        "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
         VALUES ('11111111-1111-1111-1111-111111111111', 'Item fallo', 1.0, 'ud', 'Otros', 0, 'test', CURRENT_TIMESTAMP)",
    )
    .execute(&mut *conn1)
    .await;

    match result {
        Err(sqlx::Error::Database(e)) => {
            let code = e.code().map(|c| c.to_string()).unwrap_or_default();
            let msg = e.message().to_string();
            // Comprobado empíricamente: sqlx expone el código extendido exacto
            // SQLITE_BUSY_SNAPSHOT (517) con el mensaje "database is locked".
            assert_eq!(
                code, "517",
                "se esperaba el código extendido SQLITE_BUSY_SNAPSHOT (517); code={code} msg={msg}"
            );
        }
        Ok(_) => panic!("se esperaba fallo por SQLITE_BUSY_SNAPSHOT"),
        Err(e) => panic!("error inesperado: {e}"),
    }

    sqlx::query("ROLLBACK")
        .execute(&mut *conn1)
        .await
        .expect("ROLLBACK con1");
}

#[tokio::test]
async fn immediate_begin_serializes_writers() {
    let (pool, _) = common::test_pool().await;

    let p1 = pool.clone();
    let h1 = tokio::spawn(async move {
        let mut tx = p1.begin_with("BEGIN IMMEDIATE").await.expect("begin 1");
        sqlx::query(
            "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
             VALUES ('22222222-2222-2222-2222-222222222222', 'Item 1', 1.0, 'ud', 'Otros', 0, 'test', CURRENT_TIMESTAMP)",
        )
        .execute(&mut *tx)
        .await
        .expect("insert 1");
        tx.commit().await.expect("commit 1");
    });

    let p2 = pool.clone();
    let h2 = tokio::spawn(async move {
        let mut tx = p2.begin_with("BEGIN IMMEDIATE").await.expect("begin 2");
        sqlx::query(
            "INSERT INTO shopping_list_items (id, name, amount, unit, category, checked, source_plan_id, updated_at) \
             VALUES ('33333333-3333-3333-3333-333333333333', 'Item 2', 1.0, 'ud', 'Otros', 0, 'test', CURRENT_TIMESTAMP)",
        )
        .execute(&mut *tx)
        .await
        .expect("insert 2");
        tx.commit().await.expect("commit 2");
    });

    h1.await.expect("join 1");
    h2.await.expect("join 2");

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items WHERE name IN ('Item 1', 'Item 2')",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(count, 2);
}

#[tokio::test]
async fn concurrent_writes_never_busy_snapshot() {
    let (pool, _) = common::test_pool().await;

    for _ in 0..10 {
        let mut handles = Vec::new();
        for i in 0..16 {
            let p = pool.clone();
            handles.push(tokio::spawn(async move {
                let items = vec![IngredientInput {
                    name: format!("Ingrediente Concurrente {i}"),
                    amount: 1.0,
                    unit: "ud".to_string(),
                    category: Some("Otros".to_string()),
                }];
                agent::add_shopping_items(&p, &items).await.map(|_| ())
            }));
        }
        for h in handles {
            h.await
                .expect("join")
                .expect("add_shopping_items debe ser Ok");
        }
        sqlx::query("DELETE FROM shopping_list_items")
            .execute(&pool)
            .await
            .expect("limpieza");
    }
}

/// Las tres operaciones de escritura (`set_weekly_plan`, `consolidate` y
/// `add_shopping_items`) deben serializarse sin `SQLITE_BUSY_SNAPSHOT` cuando
/// corren concurrentemente sobre el mismo pool.
#[tokio::test]
async fn concurrent_plan_and_shopping_writes_never_busy_snapshot() {
    let (pool, _) = common::test_pool().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rconc', 'Concurrente', 'main', \
                 '[{\"name\":\"Tomate\",\"amount\":2.0,\"unit\":\"ud\"}]', 2)",
    )
    .execute(&pool)
    .await
    .expect("insert receta");

    let mut handles = Vec::new();

    for i in 0..6 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            let entries = vec![StructuredPlanEntry {
                date: format!("2026-03-{:02}", i + 1),
                meal_type: "lunch".to_string(),
                recipe_id: "rconc".to_string(),
                role: "main".to_string(),
                servings: 2,
            }];
            agent::set_weekly_plan(&p, &entries).await.map(|_| ())
        }));
    }

    for _ in 0..6 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            shopping_list::consolidate(&p, "2026-03-01", "2026-03-31")
                .await
                .map(|_| ())
        }));
    }

    for i in 0..6 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            let items = vec![IngredientInput {
                name: format!("Item concurrente {i}"),
                amount: 1.0,
                unit: "ud".to_string(),
                category: Some("Otros".to_string()),
            }];
            agent::add_shopping_items(&p, &items).await.map(|_| ())
        }));
    }

    for h in handles {
        h.await
            .expect("join")
            .expect("ninguna operación debe devolver BUSY_SNAPSHOT");
    }
}
