//! Crate de OxiMeal Backend: núcleo del servidor y dominio puro.

pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod models;
pub mod repo;
pub mod routes;
pub mod services;
pub mod state;
pub mod static_files;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::Response;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};

use crate::state::AppState;

/// Construye el router completo de la aplicación con el estado inyectado.
///
/// Aplica trazabilidad de peticiones y CORS restringido a los orígenes de
/// `ALLOWED_ORIGINS` (nunca `Any` con credenciales). Cuando OIDC está activo
/// añade la capa de sesión server-side (cookie `httpOnly` firmada); las capas
/// de autorización por permiso se montan en [`routes::router`] y el `csrf_guard`
/// solo en `POST /auth/logout`. Con `OIDC_ENABLED=false` no se añade nada y el
/// comportamiento previo queda intacto.
pub fn app(state: AppState) -> Router {
    let cors = cors_layer(&state.config.allowed_origins);

    let mut router = Router::new()
        .merge(routes::router(&state))
        // El fallback del frontend debe fusionarse al final: solo actúa cuando
        // ninguna ruta de la API ha casado.
        //
        // ADVERTENCIA: `routes::router()` NO debe definir un `fallback()` propio.
        // Fusionar dos routers que aportan un `fallback` hace que axum entre en
        // pánico ("Cannot merge two `Router`s that both have a fallback"). El
        // fallback único del árbol es el de `static_files::router()`.
        .merge(static_files::router());

    // Las rutas `/auth/*` solo existen (y solo tienen sentido) con OIDC activo.
    if state.auth.is_some() {
        router = router.merge(auth::routes::router(&state));
    }

    let router = router.layer(TraceLayer::new_for_http()).layer(cors);

    // La capa de sesión debe ser la más externa para que los guards de
    // autorización y CSRF la encuentren ya resuelta en las extensiones.
    let router = match &state.auth {
        Some(auth) => {
            let session_config = &state.config.session;
            let key =
                auth::session::cookie_key(session_config.secret.as_deref().unwrap_or_default());
            let ttl = time::Duration::seconds(session_config.ttl.as_secs() as i64);
            let session_layer = SessionManagerLayer::new(auth.session_store().clone())
                .with_name(session_config.cookie_name.clone())
                .with_secure(session_config.cookie_secure)
                .with_http_only(true)
                .with_same_site(SameSite::Lax)
                .with_path("/")
                .with_expiry(Expiry::OnInactivity(ttl))
                .with_signed(key);

            router.layer(session_layer)
        }
        None => router,
    };

    // Las respuestas a peticiones con credenciales (cookie de sesión o Bearer)
    // pueden contener datos privados: nunca se cachean.
    let router = router.layer(from_fn_with_state(
        state.clone(),
        no_store_for_authenticated,
    ));

    router.with_state(state)
}

/// Indica si la petición trae credenciales (cookie de sesión o `Bearer`).
fn has_credentials(state: &AppState, headers: &HeaderMap) -> bool {
    if headers.contains_key(header::AUTHORIZATION) {
        return true;
    }

    let cookie_name = state.config.session.cookie_name.as_str();
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(|cookies| {
            cookies.split(';').any(|pair| {
                pair.trim_start()
                    .strip_prefix(cookie_name)
                    .is_some_and(|rest| rest.starts_with('='))
            })
        })
        .unwrap_or(false)
}

/// Añade `Cache-Control: no-store` a las respuestas autenticadas.
async fn no_store_for_authenticated(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let authenticated = has_credentials(&state, request.headers());
    let mut response = next.run(request).await;
    if authenticated {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

/// Construye la capa CORS a partir de la lista de orígenes permitidos.
///
/// Los orígenes inválidos se omiten con un warning. Lista vacía ⇒ ningún origen
/// autorizado. Con credenciales activadas nunca se usa `Any`.
fn cors_layer(origins: &[String]) -> CorsLayer {
    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::warn!(origin = %origin, "origen CORS inválido; se omite");
                None
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
}
