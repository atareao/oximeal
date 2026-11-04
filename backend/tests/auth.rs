//! Tests de integración del flujo BFF (bloque 2): login, callback, sesión,
//! renovación y CSRF. Usan un mock IdP con `wiremock`.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::idp::{
    ACCESS_GROUPS_REFRESH, ACCESS_NO_GROUPS_REFRESH, API_AUDIENCE, INVALID_REFRESH, MockIdp,
    NO_GROUPS_REFRESH, NO_ID_TOKEN_REFRESH, USER_GROUPS, access_token,
};
use common::{send, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::json;

/// Extrae el par `nombre=valor` de un `Set-Cookie` (sin atributos).
fn cookie_pair(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .expect("cookie no vacía")
        .trim()
        .to_string()
}

/// Devuelve la cabecera `set-cookie` de una respuesta.
fn set_cookie(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(axum::http::header::SET_COOKIE)
        .expect("debe haber Set-Cookie")
        .to_str()
        .expect("Set-Cookie válido")
        .to_string()
}

/// Devuelve la cabecera `location` de una respuesta.
fn location(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(axum::http::header::LOCATION)
        .expect("debe haber Location")
        .to_str()
        .expect("Location válido")
        .to_string()
}

/// Ejecuta `GET /auth/login{query}` y devuelve (cookie, state, nonce, location).
async fn start_login_with(state: &AppState, query: &str) -> (String, String, String, String) {
    let request = Request::builder()
        .method("GET")
        .uri(format!("/auth/login{query}"))
        .body(Body::empty())
        .expect("request login");
    let (status, headers, _) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FOUND, "login debe redirigir (302)");

    let cookie = set_cookie(&headers);
    assert!(
        cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"),
        "la cookie debe ser httpOnly y SameSite=Lax: {cookie}"
    );
    assert!(
        !cookie.to_lowercase().contains("token"),
        "la cookie no debe exponer tokens: {cookie}"
    );

    let location = location(&headers);
    let url = reqwest::Url::parse(&location).expect("Location es URL");
    let param = |key: &str| {
        url.query_pairs()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.into_owned())
            .unwrap_or_else(|| panic!("falta el parámetro {key} en {location}"))
    };
    let state_param = param("state");
    let nonce = param("nonce");
    (cookie_pair(&cookie), state_param, nonce, location)
}

/// Ejecuta `GET /auth/login` y devuelve (cookie de sesión, state, nonce, location).
async fn start_login(state: &AppState) -> (String, String, String, String) {
    start_login_with(state, "").await
}

/// Completa el flujo login+callback y devuelve la cookie de sesión autenticada.
async fn complete_login(state: &AppState) -> String {
    let (login_cookie, state_param, nonce, _location) = start_login(state).await;
    let uri = format!(
        "/auth/callback?code={nonce}&state={}",
        urlencode(&state_param)
    );
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", login_cookie)
        .body(Body::empty())
        .expect("request callback");
    let (status, headers, _) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FOUND, "callback debe redirigir");
    cookie_pair(&set_cookie(&headers))
}

/// Codifica un componente de query (state siempre es base64url, seguro, pero no asumimos).
fn urlencode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

/// Ejecuta `GET /auth/me` con la cookie dada.
async fn get_me(state: &AppState, cookie: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder().method("GET").uri("/auth/me");
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    let request = builder.body(Body::empty()).expect("request /auth/me");
    let (status, _headers, body) = send(state.clone(), request).await;
    (status, body)
}

/// Marca la sesión almacenada como caducada y con el refresh token dado.
async fn expire_session(state: &AppState, refresh_token: &str) {
    let (id, data): (String, Vec<u8>) = sqlx::query_as("SELECT id, data FROM sessions LIMIT 1")
        .fetch_one(&state.pool)
        .await
        .expect("hay una sesión guardada");

    let mut value: serde_json::Value = serde_json::from_slice(&data).expect("data es JSON");
    let auth = value
        .get_mut("state")
        .and_then(|state| state.get_mut("auth"))
        .expect("la sesión tiene auth");
    auth["access_token_expires_at"] = json!(1);
    auth["refresh_token"] = json!(refresh_token);

    let new_data = serde_json::to_vec(&value).expect("serializar data");
    sqlx::query("UPDATE sessions SET data = ?1 WHERE id = ?2")
        .bind(new_data)
        .bind(id)
        .execute(&state.pool)
        .await
        .expect("actualizar sesión");
}

#[tokio::test]
async fn login_redirects_with_pkce_state_and_nonce() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (_cookie, _state, _nonce, location) = start_login(&state).await;

    assert!(location.starts_with(&idp.authorization_endpoint));
    assert!(location.contains("response_type=code"));
    assert!(location.contains("code_challenge_method=S256"));
    assert!(location.contains("code_challenge="));
    assert!(location.contains("state="));
    assert!(location.contains("nonce="));
    assert!(location.contains("resource=https%3A%2F%2Foximeal.local%2Fapi"));
    assert!(location.contains("read%3Arecipes"));
    assert!(
        !location.contains("client_secret"),
        "la redirección no debe exponer el client_secret"
    );
}

#[tokio::test]
async fn callback_with_valid_state_creates_session() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (login_cookie, state_param, nonce, _) = start_login(&state).await;
    let uri = format!("/auth/callback?code={nonce}&state={state_param}");
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", login_cookie)
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(state, request).await;

    assert_eq!(status, StatusCode::FOUND);
    assert_eq!(location(&headers), "/");
    let cookie = set_cookie(&headers);
    assert!(cookie.contains("HttpOnly"));
}

#[tokio::test]
async fn callback_rejects_control_characters_in_return_to() {
    // #80: un `return_to` con TAB/CR se normaliza en el navegador a "//evil.com".
    // El login debe descartarlo y volver a "/" en lugar de emitir un open redirect.
    for query in ["?return_to=/%09/evil.com", "?return_to=/%0d/evil.com"] {
        let idp = MockIdp::start().await;
        let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

        let (login_cookie, state_param, nonce, _) = start_login_with(&state, query).await;
        let uri = format!(
            "/auth/callback?code={nonce}&state={}",
            urlencode(&state_param)
        );
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .header("cookie", login_cookie)
            .body(Body::empty())
            .unwrap();
        let (status, headers, _) = send(state, request).await;

        assert_eq!(status, StatusCode::FOUND);
        assert_eq!(
            location(&headers),
            "/",
            "un return_to con caracteres de control no puede escapar del origen ({query})"
        );
    }
}

#[tokio::test]
async fn callback_keeps_internal_return_to() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (login_cookie, state_param, nonce, _) =
        start_login_with(&state, "?return_to=/shopping").await;
    let uri = format!(
        "/auth/callback?code={nonce}&state={}",
        urlencode(&state_param)
    );
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", login_cookie)
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(state, request).await;

    assert_eq!(status, StatusCode::FOUND);
    assert_eq!(location(&headers), "/shopping");
}

#[tokio::test]
async fn callback_with_invalid_state_returns_400() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (login_cookie, _state_param, nonce, _) = start_login(&state).await;
    let uri = format!("/auth/callback?code={nonce}&state=definitely-wrong");
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", login_cookie)
        .body(Body::empty())
        .unwrap();
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "validation");
}

#[tokio::test]
async fn callback_outside_allowed_groups_returns_403() {
    let idp = MockIdp::start().await;
    // El ID token del mock emite `oximeal-users`; solo permitimos `admins`.
    let state = test_state_oidc(&idp.issuer, &["admins"]).await;

    let (login_cookie, state_param, nonce, _) = start_login(&state).await;
    let uri = format!("/auth/callback?code={nonce}&state={state_param}");
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", login_cookie)
        .body(Body::empty())
        .unwrap();
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn me_without_session_is_anonymous() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let (status, body) = get_me(&state, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
}

#[tokio::test]
async fn me_with_session_returns_identity_without_tokens() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    let (status, body) = get_me(&state, Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["subject"], "user-123");
    assert_eq!(body["name"], "Test User");
    assert_eq!(body["csrf_token"].as_str().map(str::is_empty), Some(false));

    let permissions = body["permissions"].as_array().expect("permisos");
    assert!(permissions.iter().any(|p| p == "read:recipes"));

    let text = body.to_string();
    assert!(
        !text.contains("access_token"),
        "no debe exponer tokens: {text}"
    );
    assert!(
        !text.contains("refresh_token"),
        "no debe exponer tokens: {text}"
    );
    assert!(!text.contains("id_token"), "no debe exponer tokens: {text}");
}

#[tokio::test]
async fn logout_destroys_session_and_clears_cookie() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    let (_status, me) = get_me(&state, Some(&cookie)).await;
    let csrf = me["csrf_token"].as_str().expect("csrf token").to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("cookie", cookie.clone())
        .header("x-csrf-token", csrf)
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(state.clone(), request).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let cleared = set_cookie(&headers);
    assert!(
        cleared.contains("Max-Age=0") || cleared.contains("oximeal_session=;"),
        "la cookie debe limpiarse: {cleared}"
    );

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
}

#[tokio::test]
async fn failed_refresh_returns_401_and_destroys_session() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    expire_session(&state, INVALID_REFRESH).await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "body: {body}");
    assert_eq!(body["error"], "unauthorized");

    // La sesión quedó destruida: con la misma cookie, ya es anónimo.
    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
}

#[tokio::test]
async fn csrf_missing_header_on_cookie_mutation_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    let request = Request::builder()
        .method("POST")
        .uri("/api/recipes")
        .header("cookie", cookie)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            }))
            .unwrap(),
        ))
        .unwrap();
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn csrf_valid_header_on_cookie_mutation_succeeds() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    let (_status, me) = get_me(&state, Some(&cookie)).await;
    let csrf = me["csrf_token"].as_str().expect("csrf token").to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/recipes")
        .header("cookie", cookie)
        .header("x-csrf-token", csrf)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            }))
            .unwrap(),
        ))
        .unwrap();
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
}

#[tokio::test]
async fn bearer_request_is_exempt_from_csrf() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    // Access token válido (write:recipes) sin cookie ni X-CSRF-Token: la
    // autorización Bearer no es *ambient authority* y queda exenta de CSRF.
    let token = access_token(&idp.issuer, "write:recipes", API_AUDIENCE);
    let request = Request::builder()
        .method("POST")
        .uri("/api/recipes")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            }))
            .unwrap(),
        ))
        .unwrap();
    let (status, _headers, body) = send(state, request).await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
}

#[tokio::test]
async fn refresh_renews_expiring_access_token_and_keeps_session() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    // Deja el access token dentro del margen con un refresh válido.
    expire_session(&state, "refresh-ok").await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["subject"], "user-123");

    // La sesión sigue viva con la misma cookie.
    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], true);
}

#[tokio::test]
async fn refresh_outside_allowed_groups_destroys_session() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    // El `id_token` renovado trae un grupo que no está en la allowlist.
    expire_session(&state, NO_GROUPS_REFRESH).await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "body: {body}");
    assert_eq!(body["error"], "unauthorized");

    // Sesión destruida: la misma cookie ya es anónima.
    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
}

#[tokio::test]
async fn refresh_groups_from_access_token_are_used_when_no_id_token() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    // Sin `id_token`: los grupos vienen en el claim `groups` del access token.
    expire_session(&state, ACCESS_GROUPS_REFRESH).await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["authenticated"], true);
}

#[tokio::test]
async fn refresh_access_token_outside_allowed_groups_destroys_session() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    expire_session(&state, ACCESS_NO_GROUPS_REFRESH).await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "body: {body}");

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
}

#[tokio::test]
async fn refresh_without_group_information_keeps_previous_groups() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    // Ni id_token ni access token verificable: se conservan los grupos previos.
    expire_session(&state, NO_ID_TOKEN_REFRESH).await;

    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["subject"], "user-123");
}

#[tokio::test]
async fn idp_down_at_startup_still_serves_health_and_401() {
    // Puerto sin servicio: el discovery fallaría, pero el router debe montarse.
    let state = test_state_oidc("http://127.0.0.1:1/", USER_GROUPS).await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();
    let (status, _headers, _body) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::OK, "health debe responder 200");

    let request = Request::builder()
        .method("GET")
        .uri("/api/recipes")
        .body(Body::empty())
        .unwrap();
    let (status, _headers, _body) = send(state, request).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "una ruta protegida sin credencial debe dar 401"
    );
}

#[tokio::test]
async fn session_persists_across_restart() {
    let idp = MockIdp::start().await;
    let (pool, database_url) = common::test_pool().await;

    // Primer "proceso": crea la sesión.
    let state = common::oidc_state(&pool, &database_url, &idp.issuer, USER_GROUPS, &[]);
    let cookie = complete_login(&state).await;

    // "Reinicio": nuevo AppState con el mismo pool/DB y el mismo secreto.
    let restarted = common::oidc_state(&pool, &database_url, &idp.issuer, USER_GROUPS, &[]);
    let (status, body) = get_me(&restarted, Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["subject"], "user-123");
}

#[tokio::test]
async fn csrf_failure_leaves_session_intact() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;

    // Mutación por cookie sin X-CSRF-Token → 403.
    let request = Request::builder()
        .method("POST")
        .uri("/api/recipes")
        .header("cookie", cookie.clone())
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "name": "Tortilla",
                "category": "main",
                "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
            }))
            .unwrap(),
        ))
        .unwrap();
    let (status, _headers, _body) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // La sesión NO se destruye: una petición posterior sigue autenticada.
    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], true);
}

#[tokio::test]
async fn logout_with_authorization_header_still_requires_csrf() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let cookie = complete_login(&state).await;
    // Bearer válido pero mutación autenticada por cookie: el logout exige CSRF
    // por cookie siempre; no hay bypass por `Authorization`.
    let token = access_token(&idp.issuer, "read:recipes", API_AUDIENCE);
    let request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("cookie", cookie.clone())
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let (status, _headers, _body) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // La sesión sigue intacta (no se destruyó por el intento sin CSRF).
    let (status, body) = get_me(&state, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], true);
}
