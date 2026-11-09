//! Tests de integración de la autorización por permisos (bloque 3).
//!
//! Cubren la validación de access tokens RFC 9068 (firma contra JWKS, `iss`,
//! `aud`, vigencia y algoritmo), la tabla de permisos por endpoint, la doble vía
//! cookie/Bearer, los endpoints públicos y el CORS restringido.

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use common::idp::{
    API_AUDIENCE, MockIdp, USER_GROUPS, access_token, access_token_with_kid, access_token_with_typ,
    alg_none_token, expired_access_token, nbf_in_future_access_token,
};
use common::{complete_login, send, test_state_oidc, test_state_oidc_full};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

/// Permiso exigido por cada petición protegida representativa.
const READ_RECIPES: &str = "read:recipes";
const WRITE_RECIPES: &str = "write:recipes";
const READ_PLAN: &str = "read:plan";
const WRITE_PLAN: &str = "write:plan";
const READ_SHOPPING: &str = "read:shopping";
const WRITE_SHOPPING: &str = "write:shopping";

/// Petición protegida representativa de cada permiso (para el escenario 401).
fn protected_requests() -> Vec<(&'static str, &'static str, Option<Value>)> {
    vec![
        ("GET", "/api/recipes", None),
        (
            "POST",
            "/api/recipes",
            Some(json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            })),
        ),
        (
            "PUT",
            "/api/recipes/missing-id",
            Some(json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            })),
        ),
        ("DELETE", "/api/recipes/missing-id", None),
        (
            "GET",
            "/api/meal-plan?start_date=2024-01-01&end_date=2024-01-31",
            None,
        ),
        ("DELETE", "/api/meal-plan/missing-id", None),
        ("GET", "/api/shopping-list", None),
        (
            "POST",
            "/api/shopping-list",
            Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
        ),
        (
            "POST",
            "/api/shopping-list/from-plan",
            Some(json!({"start_date": "2024-01-01", "end_date": "2024-01-07"})),
        ),
    ]
}

/// Envía una petición configurando credenciales, `Origin` y cuerpo opcionales.
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
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

/// Mintea un token válido con el `scope` dado contra el API del estado de test.
fn token_for(idp: &MockIdp, scope: &str) -> String {
    access_token(&idp.issuer, scope, API_AUDIENCE)
}

#[tokio::test]
async fn protected_requests_without_credentials_return_401_with_www_authenticate() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    for (method, uri, body) in protected_requests() {
        let (status, headers, _) = call(&state, method, uri, None, None, None, body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        assert_eq!(
            headers
                .get(header::WWW_AUTHENTICATE)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer"),
            "{method} {uri} debe incluir WWW-Authenticate: Bearer"
        );
    }
}

#[tokio::test]
async fn valid_token_without_required_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Token válido pero con un `scope` ajeno al permiso exigido.
    let token = token_for(&idp, "openid profile");
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    assert_eq!(body["error"], "forbidden");
}

/// Un token solo-lectura no puede actualizar ni borrar recetas → 403.
#[tokio::test]
async fn token_without_write_recipes_cannot_update_or_delete() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_RECIPES);
    let body = json!({
        "name": "Tortilla",
        "category": "main",
        "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
    });
    let (status, _headers, response) = call(
        &state,
        "PUT",
        "/api/recipes/whatever",
        Some(&token),
        None,
        None,
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {response}");

    let (status, _headers, response) = call(
        &state,
        "DELETE",
        "/api/recipes/whatever",
        Some(&token),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {response}");
}

#[tokio::test]
async fn token_with_read_recipes_permission_allows_listing() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_RECIPES);
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn token_with_write_recipes_permission_allows_creation() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, WRITE_RECIPES);
    let body = json!({
        "name": "Tortilla",
        "category": "main",
        "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
    });
    let (status, _headers, response) = call(
        &state,
        "POST",
        "/api/recipes",
        Some(&token),
        None,
        None,
        Some(body),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {response}");
}

#[tokio::test]
async fn token_with_read_plan_permission_allows_listing() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_PLAN);
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/meal-plan?start_date=2024-01-01&end_date=2024-01-31",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn token_with_write_plan_permission_allows_creation() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Prerequisito: una receta creada con un token de escritura de recetas.
    let recipe_body = json!({
        "name": "Tortilla",
        "category": "main",
        "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
    });
    let (_, _, recipe) = call(
        &state,
        "POST",
        "/api/recipes",
        Some(&token_for(&idp, WRITE_RECIPES)),
        None,
        None,
        Some(recipe_body),
    )
    .await;
    let recipe_id = recipe["id"].as_str().expect("id de receta").to_string();

    let plan_body = json!({
        "date": "2024-01-15",
        "meal_type": "lunch",
        "recipe_id": recipe_id,
        "role": "main"
    });
    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/meal-plan",
        Some(&token_for(&idp, WRITE_PLAN)),
        None,
        None,
        Some(plan_body),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
}

#[tokio::test]
async fn token_with_read_shopping_permission_allows_listing() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_SHOPPING);
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/shopping-list",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn token_with_write_shopping_permission_allows_creation() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, WRITE_SHOPPING);
    let body = json!({"name": "Leche", "amount": 1.0, "unit": "l"});
    let (status, _headers, response) = call(
        &state,
        "POST",
        "/api/shopping-list",
        Some(&token),
        None,
        None,
        Some(body),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {response}");
}

#[tokio::test]
async fn token_with_write_shopping_can_add_plan_to_list() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, WRITE_SHOPPING);
    let (status, _headers, response) = call(
        &state,
        "POST",
        "/api/shopping-list/from-plan",
        Some(&token),
        None,
        None,
        Some(json!({"start_date": "2024-01-01", "end_date": "2024-01-07"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {response}");
}

#[tokio::test]
async fn token_with_read_shopping_cannot_add_plan_to_list() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_SHOPPING);
    let (status, _headers, body) = call(
        &state,
        "POST",
        "/api/shopping-list/from-plan",
        Some(&token),
        None,
        None,
        Some(json!({"start_date": "2024-01-01", "end_date": "2024-01-07"})),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
}

#[tokio::test]
async fn token_with_unknown_kid_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token_with_kid(&idp.issuer, READ_RECIPES, API_AUDIENCE, "unknown-kid");
    let (status, _headers, _body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn jwks_is_refreshed_when_kid_is_not_cached_yet() {
    let idp = MockIdp::start_with_rotating_jwks().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // El JWKS cacheado inicial está vacío; el refresco debe traer la clave.
    let token = access_token(&idp.issuer, READ_RECIPES, API_AUDIENCE);
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn expired_token_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = expired_access_token(&idp.issuer, READ_RECIPES, API_AUDIENCE);
    let (status, _headers, _body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_with_foreign_audience_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token(&idp.issuer, READ_RECIPES, "https://other.example/api");
    let (status, _headers, _body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_with_alg_none_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = alg_none_token(&idp.issuer, READ_RECIPES, API_AUDIENCE);
    let (status, _headers, _body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_with_wrong_typ_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // `typ=JWT` (o ausente) no es un access token RFC 9068.
    for typ in [Some("JWT"), None] {
        let token = access_token_with_typ(&idp.issuer, READ_RECIPES, API_AUDIENCE, typ);
        let (status, _headers, _body) = call(
            &state,
            "GET",
            "/api/recipes",
            Some(&token),
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "typ={typ:?}");
    }
}

#[tokio::test]
async fn token_with_future_nbf_is_rejected() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = nbf_in_future_access_token(&idp.issuer, READ_RECIPES, API_AUDIENCE);
    let (status, _headers, _body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn public_endpoints_do_not_require_credentials() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    for uri in ["/api/health", "/api/openapi.json", "/api/docs"] {
        let (status, _headers, body) = call(&state, "GET", uri, None, None, None, None).await;
        assert_eq!(status, StatusCode::OK, "{uri} debe ser público: {body}");
    }
}

#[tokio::test]
async fn cookie_session_authorizes_request() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/recipes",
        None,
        Some(&cookie),
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn bearer_token_authorizes_request() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = token_for(&idp, READ_RECIPES);
    let (status, _headers, body) = call(
        &state,
        "GET",
        "/api/recipes",
        Some(&token),
        None,
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn cors_allows_configured_origin() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc_full(&idp.issuer, USER_GROUPS, &["https://app.example"]).await;

    let (status, headers, _body) = call(
        &state,
        "GET",
        "/api/health",
        None,
        None,
        Some("https://app.example"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("https://app.example")
    );
}

#[tokio::test]
async fn cors_rejects_origin_not_in_list() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc_full(&idp.issuer, USER_GROUPS, &["https://app.example"]).await;

    let (status, headers, _body) = call(
        &state,
        "GET",
        "/api/health",
        None,
        None,
        Some("https://evil.example"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none(),
        "un origen no permitido no debe autorizarse"
    );
}
