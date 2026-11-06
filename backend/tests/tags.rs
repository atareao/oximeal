//! Tests de integración de los endpoints de etiquetas (`/api/tags`).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

/// Crea una receta y devuelve su id, con las etiquetas indicadas.
async fn seed_recipe(state: &AppState, name: &str, tags: &[&str]) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "ingredients": [{"name": "Tomate", "amount": 1.0, "unit": "ud"}],
            "tags": tags,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "seed: {body}");
    body["id"].as_str().expect("id de receta").to_string()
}

/// Petición mínima con Bearer opcional para los escenarios de permisos.
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> StatusCode {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo"))
        }
        None => Body::empty(),
    };
    let (status, _headers, _value) = send(
        state.clone(),
        builder.body(body).expect("construir request"),
    )
    .await;
    status
}

#[tokio::test]
async fn list_tags_returns_all_with_recipe_count() {
    let state = test_state().await;

    seed_recipe(&state, "Con etiqueta", &["Vegano"]).await;
    // Segunda receta con la misma etiqueta para comprobar el conteo (2).
    seed_recipe(&state, "Otra con etiqueta", &["Vegano"]).await;
    // Etiqueta sin uso: se crea por su cuenta.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/tags",
        Some(json!({"name": "Sin uso"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = common::request(state, "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);

    let tags = body.as_array().expect("array de etiquetas");

    let vegano = tags
        .iter()
        .find(|t| t["slug"] == "vegano")
        .expect("etiqueta vegano");
    assert_eq!(vegano["name"], "Vegano");
    assert_eq!(vegano["recipe_count"], 2, "conteo de recetas asociadas");

    let unused = tags
        .iter()
        .find(|t| t["slug"] == "sin uso")
        .expect("etiqueta sin uso");
    assert_eq!(unused["recipe_count"], 0, "etiqueta sin recetas → 0");
}

#[tokio::test]
async fn create_tag_returns_201_with_zero_count() {
    let state = test_state().await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/tags",
        Some(json!({"name": "Desayuno"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert!(body["id"].is_string(), "debe devolver id");
    assert_eq!(body["name"], "Desayuno");
    assert_eq!(body["slug"], "desayuno");
    assert_eq!(body["recipe_count"], 0);
}

#[tokio::test]
async fn create_tag_duplicate_returns_200_with_existing() {
    let state = test_state().await;

    let (status, first) = common::request(
        state.clone(),
        "POST",
        "/api/tags",
        Some(json!({"name": "Desayuno"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let first_id = first["id"].as_str().expect("id").to_string();

    // Un nombre que normaliza al mismo slug: mayúsculas y espacios colapsados.
    let (status, second) = common::request(
        state.clone(),
        "POST",
        "/api/tags",
        Some(json!({"name": "  DESAYUNO  "})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {second}");
    assert_eq!(
        second["id"], first_id,
        "debe devolver la etiqueta existente"
    );

    let (status, body) = common::request(state, "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    let count = body
        .as_array()
        .expect("array")
        .iter()
        .filter(|t| t["slug"] == "desayuno")
        .count();
    assert_eq!(count, 1, "no debe crear una segunda etiqueta");
}

#[tokio::test]
async fn create_tag_uses_automatic_catalog_canonical_slug() {
    let state = test_state().await;

    // Una receta sin gluten hace que la heurística cree la etiqueta `auto`
    // canónica `sin-gluten`.
    let recipe_id = seed_recipe(&state, "Arroz blanco", &[]).await;
    let (status, recipe) = common::request(
        state.clone(),
        "GET",
        &format!("/api/recipes/{recipe_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "detalle de receta: {recipe}");
    let auto_tag = recipe["tags"]
        .as_array()
        .expect("tags de receta")
        .iter()
        .find(|t| t["slug"] == "sin-gluten")
        .expect("el catálogo automático debe crear sin-gluten");
    let auto_id = auto_tag["id"].as_str().expect("id auto").to_string();

    // El alta manual con el nombre de presentación resuelve al mismo `slug`
    // canónico y devuelve la etiqueta ya existente (no crea una segunda).
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/tags",
        Some(json!({"name": "Sin gluten"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["slug"], "sin-gluten");
    assert_eq!(body["id"], auto_id);

    let (status, tags) = common::request(state, "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    let count = tags
        .as_array()
        .expect("array")
        .iter()
        .filter(|t| t["slug"] == "sin-gluten")
        .count();
    assert_eq!(count, 1, "no debe duplicarse en el catálogo");
}

#[tokio::test]
async fn create_tag_empty_name_returns_400() {
    let state = test_state().await;

    for name in ["", "   ", "\t\n"] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/tags",
            Some(json!({"name": name})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "name={name:?}");
    }

    let (status, body) = common::request(state, "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("array")
            .iter()
            .all(|t| t["slug"].as_str().is_some_and(|slug| !slug.is_empty())),
        "nada con slug vacío debe persistirse: {body}"
    );
}

#[tokio::test]
async fn delete_tag_with_recipes_returns_204_and_keeps_recipes() {
    let state = test_state().await;

    let recipe_id = seed_recipe(&state, "Receta etiquetada", &["Vegano"]).await;

    let (status, tags) = common::request(state.clone(), "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    let tag_id = tags
        .as_array()
        .expect("array")
        .iter()
        .find(|t| t["slug"] == "vegano")
        .expect("etiqueta vegano")["id"]
        .as_str()
        .expect("id de etiqueta")
        .to_string();

    let (status, body) = common::request(
        state.clone(),
        "DELETE",
        &format!("/api/tags/{tag_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "body: {body}");

    // La etiqueta desaparece del catálogo...
    let (status, tags) = common::request(state.clone(), "GET", "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        tags.as_array()
            .expect("array")
            .iter()
            .all(|t| t["slug"] != "vegano")
    );

    // ...pero la receta permanece y ya no tiene la etiqueta borrada.
    let (status, detail) =
        common::request(state, "GET", &format!("/api/recipes/{recipe_id}"), None).await;
    assert_eq!(status, StatusCode::OK, "la receta debe permanecer");
    assert!(
        detail["tags"]
            .as_array()
            .expect("tags de la receta")
            .iter()
            .all(|t| t["slug"] != "vegano")
    );
}

#[tokio::test]
async fn delete_unknown_tag_returns_404() {
    let state = test_state().await;

    let (status, _) = common::request(state, "DELETE", "/api/tags/no-existe", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_tag_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let status = call(
        &state,
        "POST",
        "/api/tags",
        None,
        Some(json!({"name": "Vegano"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_tag_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token(&idp.issuer, "openid profile", API_AUDIENCE);
    let status = call(
        &state,
        "POST",
        "/api/tags",
        Some(&token),
        Some(json!({"name": "Vegano"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_tag_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let status = call(&state, "DELETE", "/api/tags/whatever", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_tag_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token(&idp.issuer, "read:recipes", API_AUDIENCE);
    let status = call(&state, "DELETE", "/api/tags/whatever", Some(&token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_tags_without_permission_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let status = call(&state, "GET", "/api/tags", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
