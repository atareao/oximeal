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
