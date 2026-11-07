//! Tests de integración de los endpoints de ajustes del hogar (`/api/settings`).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

/// Petición mínima con Bearer opcional para los escenarios de permisos.
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let request_body = match body {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo"))
        }
        None => Body::empty(),
    };
    let (status, _headers, value) = send(
        state.clone(),
        builder.body(request_body).expect("construir request"),
    )
    .await;
    (status, value)
}

#[tokio::test]
async fn get_settings_returns_seeded_value_from_migration() {
    let state = test_state().await;

    // La migración siembra la fila id=1 con default_servings=2
    let (status, body) = common::request(state, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["default_servings"], 2,
        "debe devolver el valor sembrado por la migración"
    );
    assert!(
        body["updated_at"].is_string(),
        "debe incluir updated_at del seed: {body}"
    );
}

#[tokio::test]
async fn get_settings_returns_default_2_when_row_deleted() {
    let state = test_state().await;

    // Borrar explícitamente la fila id=1 para probar el fallback a 2
    sqlx::query("DELETE FROM household_settings WHERE id = 1")
        .execute(&state.pool)
        .await
        .expect("borrar fila");

    let (status, body) = common::request(state.clone(), "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["default_servings"], 2,
        "debe devolver el valor por defecto cuando no hay fila"
    );
    assert!(
        body["updated_at"].is_null(),
        "updated_at debe ser null sin fila: {body}"
    );

    // Verificar que NO se insertó la fila
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM household_settings")
        .fetch_one(&state.pool)
        .await
        .expect("contar filas");
    assert_eq!(count, 0, "no debe insertar fila al consultar sin ella");
}

#[tokio::test]
async fn put_settings_updates_value_and_get_returns_new_value() {
    let state = test_state().await;

    // PUT con valor 4
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["default_servings"], 4,
        "debe devolver el valor actualizado"
    );

    // GET posterior debe devolver 4
    let (status, body) = common::request(state, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["default_servings"], 4, "debe persistir el valor");
}

#[tokio::test]
async fn put_settings_with_0_returns_422_and_does_not_change() {
    let state = test_state().await;

    // Establecer un valor válido inicial
    let (status, _) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 3})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Intentar PUT con 0 (inválido)
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 0})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert!(
        body["message"].as_str().is_some(),
        "debe devolver un mensaje de error"
    );

    // Verificar que el valor no cambió
    let (status, body) = common::request(state, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["default_servings"], 3,
        "no debe cambiar con PUT inválido"
    );
}

#[tokio::test]
async fn put_settings_with_21_returns_422_and_does_not_change() {
    let state = test_state().await;

    // Establecer un valor válido inicial
    let (status, _) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 5})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Intentar PUT con 21 (inválido)
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 21})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert!(
        body["message"].as_str().is_some(),
        "debe devolver un mensaje de error"
    );

    // Verificar que el valor no cambió
    let (status, body) = common::request(state, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["default_servings"], 5,
        "no debe cambiar con PUT inválido"
    );
}

#[tokio::test]
async fn get_settings_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Token sin permiso read:recipes
    let token = access_token(&idp.issuer, "openid profile", API_AUDIENCE);
    let (status, _) = call(&state, "GET", "/api/settings", Some(&token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn put_settings_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Token sin permiso write:recipes
    let token = access_token(&idp.issuer, "read:recipes", API_AUDIENCE);
    let (status, _) = call(
        &state,
        "PUT",
        "/api/settings",
        Some(&token),
        Some(json!({"default_servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_settings_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (status, _) = call(&state, "GET", "/api/settings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn put_settings_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (status, _) = call(
        &state,
        "PUT",
        "/api/settings",
        None,
        Some(json!({"default_servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// === Tests de validación y fronteras ===

#[tokio::test]
async fn put_settings_with_non_integer_string_returns_422() {
    let state = test_state().await;

    let (status, body) = common::request(
        state,
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": "cuatro"})),
    )
    .await;
    // Axum Json<T> devuelve 422 cuando la deserialización de serde falla
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "string no entero debe devolver 422: {body}"
    );
}

#[tokio::test]
async fn put_settings_with_float_returns_422() {
    let state = test_state().await;

    let (status, body) = common::request(
        state,
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 2.5})),
    )
    .await;
    // Axum Json<T> devuelve 422 cuando intenta deserializar float como i64
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "float debe devolver 422: {body}"
    );
}

#[tokio::test]
async fn put_settings_without_body_returns_415() {
    let state = test_state().await;

    let (status, body) = common::request(state.clone(), "PUT", "/api/settings", None).await;
    // Axum devuelve 415 Unsupported Media Type cuando falta Content-Type
    assert_eq!(
        status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "sin body debe devolver 415: {body}"
    );
}

#[tokio::test]
async fn put_settings_accepts_boundary_values_1_and_20() {
    let state = test_state().await;

    // Frontera inferior: 1
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 1})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "1 debe ser aceptado: {body}");
    assert_eq!(body["default_servings"], 1);

    // Frontera superior: 20
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 20})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "20 debe ser aceptado: {body}");
    assert_eq!(body["default_servings"], 20);
}

#[tokio::test]
async fn put_settings_rejects_boundary_values_0_and_21() {
    let state = test_state().await;

    // Frontera inferior violada: 0
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 0})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "0 debe ser rechazado: {body}"
    );

    // Frontera superior violada: 21
    let (status, body) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 21})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "21 debe ser rechazado: {body}"
    );
}

#[tokio::test]
async fn put_settings_persists_across_pool_recreation() {
    use oximeal_backend::config::Config;
    use std::sync::Arc;

    let (pool, database_url) = common::test_pool().await;
    let config1 = Config::for_tests(database_url.clone(), "127.0.0.1:0");
    let state1 = AppState {
        pool: pool.clone(),
        config: Arc::new(config1),
        auth: None,
    };

    // PUT con valor 7
    let (status, body) = common::request(
        state1.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 7})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["default_servings"], 7);

    // Cerrar el estado 1 y recrear con nuevo estado (simula reinicio del servidor)
    drop(state1);
    let config2 = Config::for_tests(database_url, "127.0.0.1:0");
    let state2 = AppState {
        pool,
        config: Arc::new(config2),
        auth: None,
    };

    // GET debe devolver el valor persistido
    let (status, body) = common::request(state2, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["default_servings"], 7,
        "el valor debe persistir tras recrear el pool"
    );
}

#[tokio::test]
async fn get_settings_includes_updated_at_with_iso8601_format() {
    let state = test_state().await;

    // PUT para asegurar que existe updated_at
    let (status, _) = common::request(
        state.clone(),
        "PUT",
        "/api/settings",
        Some(json!({"default_servings": 3})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = common::request(state, "GET", "/api/settings", None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let updated_at = body["updated_at"]
        .as_str()
        .expect("updated_at debe ser string");

    // Formato ISO-8601 UTC: YYYY-MM-DDTHH:MM:SS.sssZ
    let iso8601_pattern =
        regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$").expect("regex válido");
    assert!(
        iso8601_pattern.is_match(updated_at),
        "updated_at debe tener formato ISO-8601 UTC: {updated_at}"
    );
}
