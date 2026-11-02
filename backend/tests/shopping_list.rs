//! Tests de integración de los endpoints de la lista de la compra.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

/// Crea una receta con un único ingrediente y devuelve su id.
async fn create_recipe(
    state: &oximeal_backend::state::AppState,
    name: &str,
    ingredient_name: &str,
    amount: f64,
    unit: &str,
) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "ingredients": [{"name": ingredient_name, "amount": amount, "unit": unit}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().expect("id de receta").to_string()
}

/// Busca un ítem por nombre en la lista agrupada y devuelve su objeto.
fn find_item<'a>(groups: &'a Value, name: &str) -> Option<&'a Value> {
    groups.as_array()?.iter().find_map(|group| {
        group["items"]
            .as_array()?
            .iter()
            .find(|item| item["name"] == name)
    })
}

#[tokio::test]
async fn manual_create_uses_heuristic_category() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["category"], "Verdulería");
    assert_eq!(body["checked"], false);

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Tomate").expect("tomate en la lista");
    assert_eq!(item["category"], "Verdulería");
}

#[tokio::test]
async fn create_without_client_id_returns_201_and_null_client_id() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["client_id"], Value::Null);
}

#[tokio::test]
async fn create_with_new_client_id_returns_201_and_echoes_it() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["client_id"], "abc");
}

#[tokio::test]
async fn create_twice_with_same_client_id_returns_existing_without_duplicating() {
    let state = common::test_state().await;

    let (first_status, first) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (status, second) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "el alta repetida devuelve 200");
    assert_eq!(second["id"], first["id"], "devuelve el ítem existente");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE client_id = 'abc'")
            .fetch_one(&state.pool)
            .await
            .expect("contar ítems con client_id");
    assert_eq!(count, 1, "no debe duplicar el ítem");
}

#[tokio::test]
async fn concurrent_creates_with_same_client_id_create_a_single_row() {
    let state = common::test_state().await;

    let payload = json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "race"});
    let (first, second) = tokio::join!(
        common::request(
            state.clone(),
            "POST",
            "/api/shopping-list",
            Some(payload.clone())
        ),
        common::request(state.clone(), "POST", "/api/shopping-list", Some(payload)),
    );

    let statuses = [first.0, second.0];
    assert!(
        statuses.contains(&StatusCode::CREATED),
        "una de las altas debe crear el ítem: {statuses:?}"
    );
    assert!(
        statuses
            .iter()
            .all(|status| *status == StatusCode::CREATED || *status == StatusCode::OK),
        "ninguna alta debe fallar: {statuses:?}"
    );
    assert_eq!(
        first.1["id"], second.1["id"],
        "ambas respuestas apuntan al mismo ítem"
    );

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE client_id = 'race'")
            .fetch_one(&state.pool)
            .await
            .expect("contar ítems con client_id");
    assert_eq!(
        count, 1,
        "nunca debe haber dos filas con el mismo client_id"
    );
}

#[tokio::test]
async fn list_exposes_client_id() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l", "client_id": "lista"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Leche").expect("leche en la lista");
    assert_eq!(item["client_id"], "lista");
}

#[tokio::test]
async fn patch_toggles_checked() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, body) = common::request(
        state,
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["checked"], true);
}

#[tokio::test]
async fn patch_with_empty_body_returns_400() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, _) = common::request(
        state,
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn consolidate_sums_repeated_ingredients_and_keeps_manual_items() {
    let state = common::test_state().await;

    // Dos recetas con el mismo ingrediente: 2 + 3 = 5 ud de tomate.
    let r1 = create_recipe(&state, "Receta 1", "Tomate", 2.0, "ud").await;
    let r2 = create_recipe(&state, "Receta 2", "Tomate", 3.0, "ud").await;

    for payload in [
        json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"}),
        json!({"date": "2026-03-03", "meal_type": "dinner", "recipe_id": r2, "role": "main"}),
    ] {
        let (status, _) =
            common::request(state.clone(), "POST", "/api/meal-plan", Some(payload)).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    // Ítem manual que NO debe borrarse al consolidar.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let tomate = find_item(&groups, "Tomate").expect("tomate consolidado");
    assert_eq!(tomate["amount"], 5.0);
    assert_eq!(tomate["unit"], "ud");
    assert_eq!(tomate["category"], "Verdulería");

    // El ítem manual se conserva.
    assert!(
        find_item(&groups, "Leche").is_some(),
        "el ítem manual se conserva"
    );
}

#[tokio::test]
async fn consolidate_regenerates_plan_items_on_repeat() {
    let state = common::test_state().await;

    let r1 = create_recipe(&state, "Receta 1", "Tomate", 2.0, "ud").await;
    let (_, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"})),
    )
    .await;

    // Consolidar dos veces no debe duplicar las cantidades.
    for _ in 0..2 {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/consolidate",
            Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    let tomate = find_item(&groups, "Tomate").expect("tomate consolidado");
    assert_eq!(tomate["amount"], 2.0, "no debe duplicarse al reconsolidar");
}

#[tokio::test]
async fn delete_completed_leaves_only_unchecked() {
    let state = common::test_state().await;

    let (_, a) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 1.0, "unit": "ud"})),
    )
    .await;
    let id_a = a["id"].as_str().expect("id a");

    let (_, b) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id_b = b["id"].as_str().expect("id b");

    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{id_a}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = common::request(
        state.clone(),
        "DELETE",
        "/api/shopping-list/completed",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(
        find_item(&groups, "Tomate").is_none(),
        "el marcado se elimina"
    );
    assert!(
        find_item(&groups, "Leche").is_some(),
        "el no marcado permanece"
    );
    // El id_b se usa para documentar que se conserva el ítem no marcado.
    assert!(!id_b.is_empty());
}

#[tokio::test]
async fn consolidate_scales_up_by_entry_servings() {
    let state = common::test_state().await;

    // Receta por defecto (servings = 2) con 100 g.
    let r1 = create_recipe(&state, "Receta escala", "Arroz", 100.0, "g").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": r1, "role": "main", "servings": 4
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 200.0,
        "planificada a 4 raciones duplica: 100 g -> 200 g"
    );
}

#[tokio::test]
async fn consolidate_scales_down_when_recipe_has_more_servings() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Receta 4 raciones", "category": "main", "servings": 4,
            "ingredients": [{"name": "Arroz", "amount": 100.0, "unit": "g"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let r1 = body["id"].as_str().expect("id").to_string();

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": r1, "role": "main", "servings": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 50.0,
        "receta de 4 raciones planificada a 2 => mitad"
    );
}

#[tokio::test]
async fn consolidate_without_explicit_servings_keeps_amount() {
    let state = common::test_state().await;

    let r1 = create_recipe(&state, "Receta base", "Arroz", 100.0, "g").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 100.0,
        "sin raciones explícitas el factor es 1"
    );
}

#[tokio::test]
async fn consolidate_treats_null_servings_as_default_two() {
    let state = common::test_state().await;

    // Inserta con SQL directo una receta y una entrada con `servings = NULL`
    // (la API nunca persiste NULL: default 2), para ejercitar el COALESCE.
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rn', 'Receta NULL', 'main', ?, NULL)",
    )
    .bind(r#"[{"name":"Arroz","amount":100.0,"unit":"g"}]"#)
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    sqlx::query(
        "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
         VALUES ('en', '2026-03-02', 'lunch', 'rn', 'main', NULL)",
    )
    .execute(&state.pool)
    .await
    .expect("insertar entrada");

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(arroz["amount"], 100.0, "NULL/NULL => factor 1");
}

#[tokio::test]
async fn consolidate_sums_scaled_amounts_of_same_recipe() {
    let state = common::test_state().await;

    // Receta por defecto (servings = 2) con 100 g.
    let r1 = create_recipe(&state, "Receta sumada", "Arroz", 100.0, "g").await;

    for (date, servings) in [("2026-03-02", 4), ("2026-03-03", 1)] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/meal-plan",
            Some(json!({
                "date": date, "meal_type": "lunch",
                "recipe_id": r1, "role": "main", "servings": servings
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    // 100 g x (4/2) = 200 g  +  100 g x (1/2) = 50 g  =>  250 g
    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 250.0,
        "cada entrada escala por su factor y se suman"
    );
}
