//! Tests de integración de `POST /api/recipes/import`.
//!
//! Los casos de éxito usan `wiremock` como origen remoto; `common::test_state`
//! exime loopback en `import_allowed_hosts` para que la descarga sea posible.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// HTML con un bloque JSON-LD `@type: Recipe` completo.
const RECIPE_HTML: &str = r#"<!doctype html><html><head>
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Recipe",
  "name": "Tortilla de patatas",
  "recipeYield": "4",
  "prepTime": "PT30M",
  "recipeIngredient": ["4 huevos", "3 patatas"],
  "recipeInstructions": ["Pelar y cortar", "Freír"]
}
</script></head><body></body></html>"#;

/// HTML válido sin ninguna receta estructurada.
const PLAIN_HTML: &str = "<!doctype html><html><body><h1>Hola</h1></body></html>";

/// Monta un `GET <path_str>` que responde 200 con el cuerpo y `content-type`.
async fn mount_html(server: &MockServer, path_str: &str, body: &str, content_type: &str) {
    Mock::given(method("GET"))
        .and(path(path_str))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.as_bytes(), content_type))
        .mount(server)
        .await;
}

/// Importa una URL y devuelve el cuerpo JSON.
async fn import(state: AppState, url: &str) -> (StatusCode, Value) {
    common::request(
        state,
        "POST",
        "/api/recipes/import",
        Some(json!({ "url": url })),
    )
    .await
}

#[tokio::test]
async fn import_valid_recipe_returns_201_with_source_url() {
    let state = test_state().await;
    let server = MockServer::start().await;
    mount_html(&server, "/recipe", RECIPE_HTML, "text/html; charset=utf-8").await;
    let url = format!("{}/recipe", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["name"], "Tortilla de patatas");
    assert_eq!(body["source_url"], url);
    assert_eq!(body["servings"], 4);
    assert_eq!(body["prep_time_minutes"], 30);
    assert_eq!(body["ingredients"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn import_html_without_recipe_returns_422() {
    let state = test_state().await;
    let server = MockServer::start().await;
    mount_html(&server, "/page", PLAIN_HTML, "text/html; charset=utf-8").await;
    let url = format!("{}/page", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["error"], "unprocessable_entity");
}

#[tokio::test]
async fn import_origin_server_error_returns_502() {
    let state = test_state().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/broken"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let url = format!("{}/broken", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::BAD_GATEWAY, "body: {body}");
    assert_eq!(body["error"], "bad_gateway");
}

#[tokio::test]
async fn import_non_html_content_type_returns_400() {
    let state = test_state().await;
    let server = MockServer::start().await;
    mount_html(&server, "/data", "{}", "application/json").await;
    let url = format!("{}/data", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn import_malformed_url_returns_400() {
    let state = test_state().await;

    let (status, body) = import(state, "no-es-url").await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn import_oversized_body_returns_400() {
    let state = test_state().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html; charset=utf-8")
                .set_body_bytes(vec![b'a'; 2 * 1024 * 1024 + 1]),
        )
        .mount(&server)
        .await;
    let url = format!("{}/big", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn import_redirect_to_forbidden_host_returns_400() {
    let state = test_state().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("location", "http://10.0.0.1/privada"),
        )
        .mount(&server)
        .await;
    let url = format!("{}/redirect", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn import_redirect_loop_exceeding_hop_limit_returns_502() {
    let state = test_state().await;
    let server = MockServer::start().await;
    // El servidor se redirige a sí mismo indefinidamente.
    let location = format!("{}/loop", server.uri());
    Mock::given(method("GET"))
        .and(path("/loop"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", location.as_str()))
        .mount(&server)
        .await;
    let url = format!("{}/loop", server.uri());

    let (status, body) = import(state, &url).await;

    assert_eq!(status, StatusCode::BAD_GATEWAY, "body: {body}");
    assert_eq!(body["error"], "bad_gateway");
}

#[tokio::test]
async fn import_without_json_body_returns_400_with_app_envelope() {
    let state = test_state().await;

    let (status, body) = common::request(state, "POST", "/api/recipes/import", None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn reimport_same_url_is_idempotent() {
    let state = test_state().await;
    let server = MockServer::start().await;
    mount_html(&server, "/recipe", RECIPE_HTML, "text/html; charset=utf-8").await;
    let url = format!("{}/recipe", server.uri());

    let (first, first_body) = import(state.clone(), &url).await;
    assert_eq!(first, StatusCode::CREATED, "body: {first_body}");

    let (second, second_body) = import(state.clone(), &url).await;
    assert_eq!(second, StatusCode::OK, "body: {second_body}");
    assert_eq!(second_body["source_url"], url);

    let (list_status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list["total"], 1, "no debe duplicar: {list}");
}

#[tokio::test]
async fn import_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let request = json_request(
        "/api/recipes/import",
        Some("https://example.com/recipe"),
        None,
    );
    let (status, _headers, _body) = send(state, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn import_without_write_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Token válido pero solo con permiso de lectura.
    let token = access_token(&idp.issuer, "read:recipes", API_AUDIENCE);
    let request = json_request(
        "/api/recipes/import",
        Some("https://example.com/recipe"),
        Some(&token),
    );
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    assert_eq!(body["error"], "forbidden");
}

/// Construye una petición POST JSON con URL y portador opcional.
fn json_request(uri: &str, url: Option<&str>, bearer: Option<&str>) -> Request<Body> {
    let payload = match url {
        Some(url) => json!({ "url": url }),
        None => json!({}),
    };
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder
        .body(Body::from(
            serde_json::to_vec(&payload).expect("serializar cuerpo"),
        ))
        .expect("construir request")
}
