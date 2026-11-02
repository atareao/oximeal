//! Tests de integración de los endpoints de planificación.

mod common;

use axum::http::StatusCode;
use serde_json::json;

/// Crea una receta vía API y devuelve su id.
async fn create_recipe(state: &oximeal_backend::state::AppState, name: &str) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "ingredients": [{"name": "Tomate", "amount": 1.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().expect("id de receta").to_string()
}

#[tokio::test]
async fn create_valid_entry_returns_201() {
    let state = common::test_state().await;
    let recipe_id = create_recipe(&state, "Paella").await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-02-02",
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main",
            "servings": 4
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["date"], "2026-02-02");
    assert_eq!(body["meal_type"], "lunch");
    assert_eq!(body["role"], "main");
    assert_eq!(body["servings"], 4);
    assert_eq!(body["recipe"]["id"], recipe_id);
}

#[tokio::test]
async fn create_with_unknown_recipe_returns_400_and_persists_nothing() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-02-02",
            "meal_type": "lunch",
            "recipe_id": "no-existe",
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // F8: el mensaje va al cliente en castellano.
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|message| message.contains("receta no encontrada")),
        "el mensaje debe estar en castellano: {body}"
    );

    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn create_with_invalid_meal_type_returns_400() {
    let state = common::test_state().await;
    let recipe_id = create_recipe(&state, "Paella").await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-02-02",
            "meal_type": "brunch",
            "recipe_id": recipe_id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_by_range_is_ordered_by_date_and_meal_type() {
    let state = common::test_state().await;
    let a = create_recipe(&state, "Receta A").await;
    let b = create_recipe(&state, "Receta B").await;

    for payload in [
        json!({"date": "2026-02-03", "meal_type": "lunch", "recipe_id": a, "role": "main"}),
        json!({"date": "2026-02-02", "meal_type": "dinner", "recipe_id": b, "role": "main"}),
        json!({"date": "2026-02-02", "meal_type": "breakfast", "recipe_id": a, "role": "side"}),
    ] {
        let (status, _) =
            common::request(state.clone(), "POST", "/api/meal-plan", Some(payload)).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = body.as_array().expect("array de entradas");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["date"], "2026-02-02");
    assert_eq!(entries[0]["meal_type"], "breakfast");
    assert_eq!(entries[1]["date"], "2026-02-02");
    assert_eq!(entries[1]["meal_type"], "dinner");
    assert_eq!(entries[2]["date"], "2026-02-03");
    assert_eq!(entries[2]["meal_type"], "lunch");
}

#[tokio::test]
async fn invalid_range_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-28&end_date=2026-02-01",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_entry_returns_204_and_removes_it() {
    let state = common::test_state().await;
    let recipe_id = create_recipe(&state, "Paella").await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-02-02",
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().expect("id de entrada").to_string();

    let (status, _) = common::request(
        state.clone(),
        "DELETE",
        &format!("/api/meal-plan/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(body.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn delete_unknown_entry_returns_404() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "DELETE", "/api/meal-plan/no-existe", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// F2 — Una fecha no rellenada (`2026-1-2`) debe persistirse canónica
/// (`2026-01-02`) para no romper los rangos `WHERE date >= ... <= ...`.
#[tokio::test]
async fn create_with_non_padded_date_persists_canonical_date() {
    let state = common::test_state().await;
    let recipe_id = create_recipe(&state, "Paella").await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-1-2",
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["date"], "2026-01-02", "la fecha debe normalizarse");

    // El rango de enero debe devolverla aunque se enviara sin rellenar.
    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-01-01&end_date=2026-01-31",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = body.as_array().expect("array de entradas");
    assert_eq!(entries.len(), 1, "la entrada debe aparecer en el rango");
    assert_eq!(entries[0]["date"], "2026-01-02");
}
