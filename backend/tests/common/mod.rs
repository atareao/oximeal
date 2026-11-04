//! Helpers compartidos por los tests de integración.
//!
//! Crean una base de datos SQLite temporal por test, aplican las migraciones
//! y construyen un [`AppState`] listo para usar con el router de `app()`.

#![allow(dead_code)]

pub mod idp;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use http_body_util::BodyExt;
use oximeal_backend::auth::AuthService;
use oximeal_backend::config::{Config, OidcConfig, SessionConfig};
use oximeal_backend::db;
use oximeal_backend::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

/// Crea una base de datos SQLite temporal con las migraciones aplicadas.
///
/// El `TempDir` se filtra deliberadamente (`mem::forget`) para que el fichero
/// no se borre mientras el pool sigue abierto; es un proceso de test efímero.
pub async fn test_state() -> AppState {
    let (pool, database_url) = test_pool().await;

    let config = Config::for_tests(database_url, "127.0.0.1:0");

    AppState {
        pool,
        config: Arc::new(config),
        auth: None,
    }
}

/// Construye un [`AppState`] con OIDC activado apuntando a un issuer (mock).
///
/// `allowed_groups` es la allowlist del backend. El cliente y la sesión usan
/// valores de test deterministas; la cookie no es `Secure` para simplificar.
pub async fn test_state_oidc(issuer: &str, allowed_groups: &[&str]) -> AppState {
    test_state_oidc_full(issuer, allowed_groups, &[]).await
}

/// Variante de [`test_state_oidc`] con `post_logout_redirect_uri` configurable.
pub async fn test_state_oidc_full(
    issuer: &str,
    allowed_groups: &[&str],
    allowed_origins: &[&str],
) -> AppState {
    let (pool, database_url) = test_pool().await;
    oidc_state(
        &pool,
        &database_url,
        issuer,
        allowed_groups,
        allowed_origins,
    )
}

/// Construye un [`AppState`] OIDC sobre un pool ya existente.
///
/// Permite simular un "reinicio" del servidor reutilizando la misma base de
/// datos y el mismo secreto de sesión (con lo que la cookie previa sigue siendo
/// válida) pero con un `AppState` nuevo.
pub fn oidc_state(
    pool: &sqlx::SqlitePool,
    database_url: &str,
    issuer: &str,
    allowed_groups: &[&str],
    allowed_origins: &[&str],
) -> AppState {
    let config = Config {
        database_url: database_url.to_string(),
        bind_addr: "127.0.0.1:0".to_string(),
        allowed_origins: allowed_origins.iter().map(|s| s.to_string()).collect(),
        import_allowed_hosts: Vec::new(),
        session: SessionConfig {
            secret: Some("integration-tests-session-secret-value".to_string()),
            ttl: Duration::from_secs(259_200),
            refresh_margin: Duration::from_secs(300),
            cookie_name: "oximeal_session".to_string(),
            cookie_secure: false,
        },
        oidc: Some(OidcConfig {
            issuer: issuer.to_string(),
            client_id: "oximeal-spa".to_string(),
            client_secret: "test-client-secret".to_string(),
            api_audience: "https://oximeal.local/api".to_string(),
            redirect_uri: "http://localhost/auth/callback".to_string(),
            post_logout_redirect_uri: "http://localhost/".to_string(),
            allowed_groups: allowed_groups.iter().map(|s| s.to_string()).collect(),
        }),
    };

    let auth = AuthService::build(&config, pool.clone())
        .expect("construir AuthService de test")
        .expect("OIDC debe estar activo");

    AppState {
        pool: pool.clone(),
        config: Arc::new(config),
        auth: Some(auth),
    }
}

/// Crea el pool temporal sobre un directorio que se conserva vivo.
pub async fn test_pool() -> (sqlx::SqlitePool, String) {
    let dir = tempfile::tempdir().expect("crear tempdir");
    let db_path = dir.path().join("test.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let pool = db::init_pool(&database_url)
        .await
        .expect("inicializar pool de test");

    // Mantener vivo el directorio temporal durante la vida del proceso de test.
    std::mem::forget(dir);

    (pool, database_url)
}

/// Envía una petición al router completo y devuelve el estado HTTP y el JSON.
///
/// Si `body` es `Some`, se serializa como JSON y se fija la cabecera
/// `content-type`. Si la respuesta no tiene cuerpo (p. ej. 204), devuelve
/// `serde_json::Value::Null`.
#[allow(dead_code)]
pub async fn request(
    state: AppState,
    method: &str,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo JSON"))
        }
        None => Body::empty(),
    };
    let request = builder.body(request_body).expect("construir request");

    let (status, _headers, value) = send(state, request).await;
    (status, value)
}

/// Extrae el par `nombre=valor` de una cabecera `Set-Cookie` (sin atributos).
pub fn cookie_pair(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .expect("cookie no vacía")
        .trim()
        .to_string()
}

/// Codifica un componente de query string (para el `state` del callback).
fn urlencode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

/// Completa el flujo BFF (login + callback) y devuelve la cookie autenticada.
///
/// Se apoya en el contrato del mock IdP: el `code` del callback transporta el
/// `nonce` que firma el ID token. Requiere un [`AppState`] con OIDC activo.
pub async fn complete_login(state: &AppState) -> String {
    let request = Request::builder()
        .method("GET")
        .uri("/auth/login")
        .body(Body::empty())
        .expect("request login");
    let (status, headers, _) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FOUND, "login debe redirigir");

    let login_cookie = headers
        .get(axum::http::header::SET_COOKIE)
        .expect("Set-Cookie del login")
        .to_str()
        .expect("Set-Cookie válido")
        .to_string();
    let location = headers
        .get(axum::http::header::LOCATION)
        .expect("Location del login")
        .to_str()
        .expect("Location válido")
        .to_string();

    let url = reqwest::Url::parse(&location).expect("Location es URL");
    let param = |key: &str| {
        url.query_pairs()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.into_owned())
            .unwrap_or_else(|| panic!("falta el parámetro {key} en {location}"))
    };
    let state_param = param("state");
    let nonce = param("nonce");

    let uri = format!(
        "/auth/callback?code={nonce}&state={}",
        urlencode(&state_param)
    );
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("cookie", cookie_pair(&login_cookie))
        .body(Body::empty())
        .expect("request callback");
    let (status, headers, _) = send(state.clone(), request).await;
    assert_eq!(status, StatusCode::FOUND, "callback debe redirigir");

    let set_cookie = headers
        .get(axum::http::header::SET_COOKIE)
        .expect("Set-Cookie del callback")
        .to_str()
        .expect("Set-Cookie válido")
        .to_string();
    cookie_pair(&set_cookie)
}

/// Envía una petición ya construida y devuelve estado, cabeceras y cuerpo JSON.
pub async fn send(
    state: AppState,
    request: Request<Body>,
) -> (StatusCode, HeaderMap, serde_json::Value) {
    let response = oximeal_backend::app(state)
        .oneshot(request)
        .await
        .expect("ejecutar request");

    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes();
    let value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };

    (status, headers, value)
}
