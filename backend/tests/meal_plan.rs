//! Tests de integración de los endpoints de planificación.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, access_token};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

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

/// Scenario "Raciones por defecto": sin `servings` en el cuerpo, la entrada se
/// crea con `servings` igual a `2` y así se persiste.
#[tokio::test]
async fn create_without_servings_defaults_to_two_and_persists() {
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
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["servings"], 2, "las raciones por defecto deben ser 2");

    // La entrada persiste con el valor por defecto.
    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = body.as_array().expect("array de entradas");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["servings"], 2);
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

#[tokio::test]
async fn patch_servings_updates_entry_preserving_identity() {
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
            "role": "main",
            "servings": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().expect("id de entrada").to_string();

    let (status, body) = common::request(
        state,
        "PATCH",
        &format!("/api/meal-plan/{id}"),
        Some(json!({"servings": 4})),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["servings"], 4);
    // Conserva identidad, receta y slot.
    assert_eq!(body["id"], id);
    assert_eq!(body["recipe"]["id"], recipe_id);
    assert_eq!(body["date"], "2026-02-02");
    assert_eq!(body["meal_type"], "lunch");
    assert_eq!(body["role"], "main");
}

#[tokio::test]
async fn patch_with_servings_below_one_returns_400_and_keeps_servings() {
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
            "role": "main",
            "servings": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().expect("id de entrada").to_string();

    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/meal-plan/{id}"),
        Some(json!({"servings": 0})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Las raciones no cambian.
    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = body.as_array().expect("array de entradas");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["servings"], 2, "las raciones deben permanecer");
}

#[tokio::test]
async fn patch_without_servings_returns_400() {
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
        state,
        "PATCH",
        &format!("/api/meal-plan/{id}"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_with_malformed_body_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "PATCH",
        "/api/meal-plan/whatever",
        Some(json!({"servings": "muchas"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_unknown_entry_returns_404() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "PATCH",
        "/api/meal-plan/no-existe",
        Some(json!({"servings": 3})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Envía una petición con token Bearer opcional (patrón de `authorization.rs`).
async fn call_with_token(
    state: &AppState,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo"))
        }
        None => Body::empty(),
    };
    let request = builder.body(body).expect("construir request");
    let (status, _headers, value) = common::send(state.clone(), request).await;
    (status, value)
}

#[tokio::test]
async fn plan_writes_without_credentials_return_401() {
    let idp = MockIdp::start().await;
    let state = common::test_state_oidc(&idp.issuer, common::idp::USER_GROUPS).await;

    let requests = [
        ("POST", "/api/meal-plan"),
        ("PATCH", "/api/meal-plan/missing-id"),
        ("DELETE", "/api/meal-plan/missing-id"),
    ];

    for (method, uri) in requests {
        let (status, _) =
            call_with_token(&state, method, uri, None, Some(json!({"servings": 2}))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }
}

#[tokio::test]
async fn plan_writes_without_write_permission_return_403() {
    let idp = MockIdp::start().await;
    let state = common::test_state_oidc(&idp.issuer, common::idp::USER_GROUPS).await;
    let token = access_token(&idp.issuer, "read:plan", API_AUDIENCE);

    let requests = [
        ("POST", "/api/meal-plan"),
        ("PATCH", "/api/meal-plan/missing-id"),
        ("DELETE", "/api/meal-plan/missing-id"),
    ];

    for (method, uri) in requests {
        let (status, body) = call_with_token(
            &state,
            method,
            uri,
            Some(&token),
            Some(json!({"servings": 2})),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}: {body}");
    }
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

/// Crea una receta con etiquetas vía API y devuelve su id.
async fn create_recipe_with_tags(state: &AppState, name: &str, tags: &[&str]) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "ingredients": [{"name": "Tomate", "amount": 1.0, "unit": "ud"}],
            "tags": tags
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "seed: {body}");
    body["id"].as_str().expect("id de receta").to_string()
}

/// El resumen de la receta embebida en una entrada del plan incluye sus
/// etiquetas (`id`, `name`, `slug`) al listar el plan por rango de fechas.
#[tokio::test]
async fn list_entries_include_recipe_tags() {
    let state = common::test_state().await;
    let recipe_id = create_recipe_with_tags(&state, "Salteado", &["Vegano", "Rápido"]).await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-02-05",
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    let (status, body) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-02-01&end_date=2026-02-28",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = body.as_array().expect("array de entradas");
    assert_eq!(entries.len(), 1);

    let recipe = &entries[0]["recipe"];
    assert_eq!(recipe["id"], recipe_id);

    let tags = recipe["tags"].as_array().expect("array de etiquetas");
    assert_eq!(tags.len(), 2, "la receta embebida devuelve sus etiquetas");

    // Cada etiqueta expone `id`, `name` y `slug`.
    for tag in tags {
        assert!(
            tag["id"].as_str().is_some_and(|id| !id.is_empty()),
            "id de etiqueta: {tag}"
        );
        assert!(tag["name"].is_string(), "name de etiqueta: {tag}");
        assert!(tag["slug"].is_string(), "slug de etiqueta: {tag}");
    }

    // Comprobación por conjunto (orden por nombre): slugs y nombres esperados.
    let mut got: Vec<(String, String)> = tags
        .iter()
        .map(|tag| {
            (
                tag["slug"].as_str().expect("slug").to_string(),
                tag["name"].as_str().expect("name").to_string(),
            )
        })
        .collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            ("rápido".to_string(), "Rápido".to_string()),
            ("vegano".to_string(), "Vegano".to_string()),
        ]
    );
}
