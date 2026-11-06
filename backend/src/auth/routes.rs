//! Rutas públicas del flujo de autenticación BFF.
//!
//! `GET /auth/login` → `GET /auth/callback` → `GET /auth/me` / `POST /auth/logout`.
//! Estas rutas no exigen credencial; el navegador solo maneja la cookie opaca.

use axum::extract::{Query, Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tower_sessions::Session;

use crate::state::AppState;

use super::error::{AuthError, AuthResult};
use super::session::{AuthenticatedSession, PreAuthFlow, SESSION_KEY, SessionState};
use super::{
    AuthService, BASE_SCOPES, MeResponse, SPA_PERMISSIONS, code_challenge_s256, csrf,
    permissions_of, random_token, sanitize_return_to,
};

/// Parámetros de `GET /auth/login`.
#[derive(Debug, Deserialize)]
pub struct LoginParams {
    /// Ruta interna a la que volver tras el login.
    #[serde(default)]
    pub return_to: Option<String>,
}

/// Parámetros de `GET /auth/callback`.
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    /// Código de autorización.
    pub code: String,
    /// Valor `state` devuelto por el proveedor.
    pub state: String,
}

/// Router de las rutas `/auth/*` (solo se monta con OIDC activado).
///
/// `POST /auth/logout` es la única mutación autenticada por cookie fuera de la
/// tabla de permisos, por lo que conserva el `csrf_guard` del bloque 2. Las
/// rutas de la API aplican su propio CSRF dentro de
/// [`crate::auth::middleware::require_permission`], evitando una doble
/// comprobación.
pub fn router(state: &AppState) -> Router<AppState> {
    let logout = Router::new()
        .route("/auth/logout", post(logout))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            csrf_guard,
        ));

    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/me", get(me))
        .merge(logout)
}

/// Accede al servicio de autenticación o falla si no está configurado.
fn service(state: &AppState) -> AuthResult<&std::sync::Arc<AuthService>> {
    state
        .auth
        .as_ref()
        .ok_or_else(|| AuthError::Internal("autenticación no configurada".into()))
}

/// Mapea errores del almacén de sesiones de `tower-sessions`.
fn session_error(error: tower_sessions::session::Error) -> AuthError {
    AuthError::store(error)
}

/// Construye una respuesta `302 Found` con la cabecera `Location`.
///
/// Se hace a mano porque `axum::response::Redirect::to` responde 303.
fn redirect_found(url: &str) -> AuthResult<Response> {
    let location =
        HeaderValue::from_str(url).map_err(|_| AuthError::Internal("Location inválida".into()))?;
    Ok((StatusCode::FOUND, [(header::LOCATION, location)]).into_response())
}

/// `GET /auth/login` — inicia el flujo Authorization Code + PKCE y redirige.
#[utoipa::path(
    get,
    path = "/auth/login",
    tag = "auth",
    params(("return_to" = Option<String>, Query, description = "Ruta interna de retorno")),
    responses((status = 302, description = "Redirección al proveedor de identidad")),
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<LoginParams>,
) -> AuthResult<Response> {
    let auth = service(&state)?;
    let discovery = auth.oidc().discovery().await?;

    let flow_state = random_token(32);
    let nonce = random_token(32);
    let code_verifier = random_token(32);
    let code_challenge = code_challenge_s256(&code_verifier);

    // Conserva una sesión previa (si la hubiera) pero sustituye el flujo.
    let mut session_state: SessionState = session
        .get(SESSION_KEY)
        .await
        .map_err(session_error)?
        .unwrap_or_default();
    session_state.flow = Some(PreAuthFlow {
        state: flow_state.clone(),
        nonce: nonce.clone(),
        code_verifier,
        return_to: sanitize_return_to(params.return_to),
    });

    // Rotación preventiva del identificador si ya existía una sesión.
    if session.id().is_some() {
        session.cycle_id().await.map_err(session_error)?;
    }
    session
        .insert(SESSION_KEY, &session_state)
        .await
        .map_err(session_error)?;

    let scope = format!("{} {}", BASE_SCOPES, SPA_PERMISSIONS.join(" "));
    let query = [
        ("response_type", "code"),
        ("client_id", auth.oidc_config().client_id.as_str()),
        ("redirect_uri", auth.oidc_config().redirect_uri.as_str()),
        ("scope", scope.as_str()),
        ("resource", auth.oidc_config().api_audience.as_str()),
        ("state", flow_state.as_str()),
        ("nonce", nonce.as_str()),
        ("code_challenge", code_challenge.as_str()),
        ("code_challenge_method", "S256"),
    ];

    let url = auth
        .oidc()
        .build_authorize_url(&discovery.authorization_endpoint, &query)?;
    redirect_found(&url)
}

/// `GET /auth/callback` — valida `state`, intercambia el código y crea sesión.
#[utoipa::path(
    get,
    path = "/auth/callback",
    tag = "auth",
    params(
        ("code" = String, Query, description = "Código de autorización"),
        ("state" = String, Query, description = "Valor state devuelto por el IdP"),
    ),
    responses(
        (status = 302, description = "Redirección al destino interno"),
        (status = 400, description = "State inválido o login no iniciado"),
        (status = 403, description = "Usuario fuera de los grupos permitidos"),
    ),
)]
pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> AuthResult<Response> {
    let auth = service(&state)?;

    let session_state: SessionState = session
        .get(SESSION_KEY)
        .await
        .map_err(session_error)?
        .ok_or_else(|| AuthError::BadRequest("no hay login en curso".into()))?;
    let flow = session_state
        .flow
        .ok_or_else(|| AuthError::BadRequest("no hay login en curso".into()))?;

    // Comparación en tiempo constante del `state` contra el guardado.
    if !csrf::verify(&flow.state, Some(&params.state)) {
        return Err(AuthError::BadRequest("state inválido".into()));
    }

    let oidc_config = auth.oidc_config();
    let tokens = auth
        .oidc()
        .exchange_code(
            &params.code,
            &oidc_config.redirect_uri,
            &oidc_config.client_id,
            &oidc_config.client_secret,
            &flow.code_verifier,
        )
        .await?;

    let id_token = tokens.id_token.clone().ok_or(AuthError::Unauthorized)?;

    let claims = auth
        .oidc()
        .verify_id_token(&id_token, Some(&flow.nonce), &oidc_config.client_id)
        .await?;

    if !auth.groups_allowed(&claims.groups) {
        return Err(AuthError::Forbidden);
    }

    let now = chrono::Utc::now().timestamp();
    let authenticated = AuthenticatedSession {
        subject: claims.sub,
        name: claims.name,
        email: claims.email,
        groups: claims.groups,
        permissions: permissions_of(&tokens),
        csrf_token: random_token(32),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        id_token: Some(id_token),
        access_token_expires_at: now + tokens.expires_in.unwrap_or(3600),
        started_at: now,
    };

    let return_to = flow.return_to.unwrap_or_else(|| "/".to_string());
    let new_state = SessionState {
        flow: None,
        auth: Some(authenticated),
    };

    // Rotación del identificador de sesión al autenticar (anti-fijación).
    session.cycle_id().await.map_err(session_error)?;
    session
        .insert(SESSION_KEY, &new_state)
        .await
        .map_err(session_error)?;

    redirect_found(&return_to)
}

/// `GET /auth/me` — estado de la sesión; nunca expone tokens.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Estado de autenticación", body = MeResponse),
        (status = 401, description = "Sesión caducada e irrenovable"),
    ),
)]
pub async fn me(State(state): State<AppState>, session: Session) -> AuthResult<Response> {
    let auth = service(&state)?;

    let mut session_state: SessionState =
        match session.get(SESSION_KEY).await.map_err(session_error)? {
            Some(session_state) => session_state,
            None => return Ok(Json(MeResponse::anonymous()).into_response()),
        };

    if session_state.auth.is_none() {
        return Ok(Json(MeResponse::anonymous()).into_response());
    }

    // Vida absoluta: se mide desde el login y el refresco no la extiende.
    let absolute_ttl = state.config.session.absolute_ttl;
    if let Some(authenticated) = session_state.auth.as_ref()
        && authenticated.is_past_absolute_ttl(absolute_ttl)
    {
        if session.flush().await.is_err() {
            tracing::warn!("no se pudo destruir la sesión caducada");
        }
        return Ok(Json(MeResponse::anonymous()).into_response());
    }

    let changed = match auth.ensure_fresh(&mut session_state).await {
        Ok(changed) => changed,
        Err(error) => {
            // Refresh fallido o usuario ya no autorizado: se destruye la sesión.
            if session.flush().await.is_err() {
                tracing::warn!("no se pudo destruir la sesión tras un fallo de renovación");
            }
            return Err(error);
        }
    };

    if changed {
        session
            .insert(SESSION_KEY, &session_state)
            .await
            .map_err(session_error)?;
    }

    match session_state.auth.as_ref() {
        Some(authenticated) => Ok(Json(MeResponse::authenticated(authenticated)).into_response()),
        None => Ok(Json(MeResponse::anonymous()).into_response()),
    }
}

/// `POST /auth/logout` — destruye la sesión, limpia la cookie y responde 204.
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses((status = 204, description = "Sesión cerrada")),
)]
pub async fn logout(State(state): State<AppState>, session: Session) -> AuthResult<Response> {
    let _auth = service(&state)?;
    session.flush().await.map_err(session_error)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Middleware anti-CSRF para mutaciones autenticadas por cookie.
///
/// Solo actúa si OIDC está activo, el método es mutante y existe una sesión
/// cookie autenticada; en ese caso exige `X-CSRF-Token` coincidente y responde
/// 403 dejando la sesión intacta si falta o no coincide.
///
/// A diferencia de la autorización de la API, aquí **no** se exime por la
/// presencia de `Authorization`: `POST /auth/logout` es una operación del SPA
/// con cookie, y el token anti-CSRF se exige siempre que haya sesión cookie.
pub async fn csrf_guard(
    State(state): State<AppState>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let mutating = matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if state.auth.is_some() && mutating {
        match session.get::<SessionState>(SESSION_KEY).await {
            Ok(Some(session_state)) => {
                if let Some(auth) = session_state.auth.as_ref() {
                    let provided = request
                        .headers()
                        .get("x-csrf-token")
                        .and_then(|value| value.to_str().ok());
                    if !csrf::verify(&auth.csrf_token, provided) {
                        return AuthError::Forbidden.into_response();
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                tracing::error!(error = %error, "no se pudo leer la sesión en el guard CSRF");
                return AuthError::store(error).into_response();
            }
        }
    }

    next.run(request).await
}
