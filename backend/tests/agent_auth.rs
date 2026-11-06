//! Tests de autorización de la Agent API M2M (bloque 4).
//!
//! Verifican la tabla de permisos `agent:read`, `agent:plan` y `agent:shopping`
//! sobre `/api/agent/*` con OIDC activo: los tokens M2M con el permiso acceden,
//! los tokens user-delegated (sin `agent:*`) reciben 403 y un permiso ajeno
//! (p. ej. `read:recipes`) no se hereda. Además comprueban que el contrato de
//! respuestas no cambia y que no se acepta ningún API key estático.

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

/// Permiso M2M de lectura de la Agent API.
const AGENT_READ: &str = "agent:read";
/// Permiso M2M de planificación de la Agent API.
const AGENT_PLAN: &str = "agent:plan";
/// Permiso M2M de lista de la compra de la Agent API.
const AGENT_SHOPPING: &str = "agent:shopping";
/// Permiso M2M de creación de recetas de la Agent API.
const AGENT_WRITE: &str = "agent:write";

/// Mintea un access token válido con el `scope` dado contra el API de test.
fn token_for(idp: &MockIdp, scope: &str) -> String {
    access_token(&idp.issuer, scope, API_AUDIENCE)
}

/// Envía una petición con `Bearer`, `Cookie` y cuerpo JSON opcionales.
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let body = match body {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo"))
        }
        None => Body::empty(),
    };
    let request = builder.body(body).expect("construir request");
    send(state.clone(), request).await
}

/// Los cinco endpoints de la Agent API con una petición representativa.
fn agent_requests() -> Vec<(&'static str, &'static str, Option<Value>)> {
    vec![
        ("GET", "/api/agent/recent-recipes", None),
        ("GET", "/api/agent/search-recipes?query=tomate", None),
        (
            "POST",
            "/api/agent/weekly-plan",
            Some(json!({"entries": []})),
        ),
        (
            "POST",
            "/api/agent/shopping-items",
            Some(json!({"items": [{"name": "Leche", "amount": 1.0, "unit": "l"}]})),
        ),
        ("DELETE", "/api/agent/shopping/completed", None),
    ]
}

// --- 4.2 Permisos dedicados por endpoint -----------------------------------

#[tokio::test]
async fn m2m_token_with_agent_read_accesses_read_endpoints() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_READ);

    for uri in ["/api/agent/recent-recipes", "/api/agent/search-recipes"] {
        let (status, _headers, body) = call(&state, "GET", uri, Some(&token), None, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
    }
}

#[tokio::test]
async fn m2m_token_with_agent_plan_accesses_weekly_plan() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_PLAN);

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/weekly-plan",
        Some(&token),
        None,
        Some(json!({"entries": []})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn m2m_token_with_agent_shopping_accesses_shopping_endpoints() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_SHOPPING);

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/shopping-items",
        Some(&token),
        None,
        Some(json!({"items": [{"name": "Leche", "amount": 1.0, "unit": "l"}]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (status, _headers, body) = call(
        &state,
        "DELETE",
        "/api/agent/shopping/completed",
        Some(&token),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["removed"], 0);
}

#[tokio::test]
async fn m2m_token_with_agent_write_creates_recipe() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_WRITE);

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/recipes",
        Some(&token),
        None,
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert!(body["id"].is_string(), "debe devolver id: {body}");
}

#[tokio::test]
async fn m2m_token_without_agent_write_is_forbidden() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_READ);

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/recipes",
        Some(&token),
        None,
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
}

#[tokio::test]
async fn user_token_cannot_create_agent_recipe() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    // Token de usuario SPA: permisos de dominio, ninguno `agent:*`.
    let token = token_for(&idp, "openid read:recipes write:recipes");

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/recipes",
        Some(&token),
        None,
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
}

#[tokio::test]
async fn agent_recipe_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (status, headers, _body) = call(
        &state,
        "POST",
        "/api/agent/recipes",
        None,
        None,
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        headers
            .get(header::WWW_AUTHENTICATE)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer")
    );
}

#[tokio::test]
async fn user_delegated_token_without_agent_permissions_is_forbidden() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    // Token de usuario SPA: permisos de dominio, ninguno `agent:*`.
    let token = token_for(&idp, "openid read:recipes write:recipes");

    for (method, uri, body) in agent_requests() {
        let (status, _headers, response) =
            call(&state, method, uri, Some(&token), None, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}: {response}");
    }
}

#[tokio::test]
async fn read_recipes_permission_does_not_grant_agent_permissions() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, "read:recipes");

    for (method, uri, body) in agent_requests() {
        let (status, _headers, response) =
            call(&state, method, uri, Some(&token), None, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{method} {uri} no debe heredar agent:*: {response}"
        );
    }
}

#[tokio::test]
async fn agent_endpoints_without_credentials_return_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    for (method, uri, body) in agent_requests() {
        let (status, headers, _body) = call(&state, method, uri, None, None, body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        assert_eq!(
            headers
                .get(header::WWW_AUTHENTICATE)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer"),
            "{method} {uri} debe anunciar WWW-Authenticate: Bearer"
        );
    }
}

// --- 4.3 Contrato sin cambios ----------------------------------------------

#[tokio::test]
async fn recent_recipes_response_schema_is_unchanged_with_authorized_token() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;
    let token = token_for(&idp, AGENT_READ);

    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/agent/recent-recipes",
        Some(&token),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let object = body.as_object().expect("objeto de respuesta");
    assert_eq!(object.len(), 1, "el esquema solo expone `recipes`: {body}");
    let recipes = object["recipes"].as_array().expect("array recipes");
    assert!(recipes.is_empty(), "sin planificación la lista va vacía");
}

#[tokio::test]
async fn weekly_plan_response_schema_is_unchanged_with_authorized_token() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Prerequisito: una receta creada con permiso de escritura de recetas.
    let recipe_body = json!({
        "name": "Tortilla",
        "category": "main",
        "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
    });
    let (status, _headers, recipe) = call(
        &state,
        "POST",
        "/api/recipes",
        Some(&token_for(&idp, "write:recipes")),
        None,
        Some(recipe_body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {recipe}");
    let recipe_id = recipe["id"].as_str().expect("id de receta").to_string();

    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/agent/weekly-plan",
        Some(&token_for(&idp, AGENT_PLAN)),
        None,
        Some(json!({"entries": [
            {"date": "2024-01-15", "meal_type": "lunch", "recipe_id": recipe_id, "role": "main"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let object = body.as_object().expect("objeto de respuesta");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["created", "shopping_list"],
        "el esquema de `weekly-plan` no debe cambiar: {body}"
    );
    assert_eq!(object["created"], 1);
    assert!(object["shopping_list"].is_array());
}

// --- 4.4 Retirada del API key estático -------------------------------------

#[tokio::test]
async fn static_api_key_is_not_accepted_as_credential() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Esquema no-Bearer en `Authorization`: no es una credencial OIDC válida.
    let mut builder = Request::builder()
        .method("GET")
        .uri("/api/agent/recent-recipes")
        .header(header::AUTHORIZATION, "ApiKey old-static-key");
    let request = builder.body(Body::empty()).expect("request api key");
    let (status, _headers, _body) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Cabecera dedicada de API key: se ignora y sin sesión ⇒ 401.
    builder = Request::builder()
        .method("GET")
        .uri("/api/agent/recent-recipes")
        .header("x-api-key", "old-static-key");
    let request = builder.body(Body::empty()).expect("request x-api-key");
    let (status, _headers, _body) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
