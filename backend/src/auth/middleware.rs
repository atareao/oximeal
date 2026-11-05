//! Middleware de autorización por permiso (doble vía cookie/Bearer).
//!
//! [`require_permission`] protege un router exigiendo un permiso concreto
//! (`read:recipes`, `write:plan`, `agent:plan`, …).
//!
//! Reglas:
//!
//! 1. `OIDC_ENABLED=false` (`state.auth == None`) ⇒ **passthrough**: el modo
//!    abierto de desarrollo no aplica ninguna comprobación.
//! 2. Si hay `Authorization: Bearer <token>` se valida el access token
//!    (RFC 9068) y sus permisos salen del claim `scope`. Si no, se usa la
//!    cookie de sesión y los permisos guardados (tras `ensure_fresh`).
//! 3. Sin credencial válida ⇒ **401** con `WWW-Authenticate: Bearer`.
//! 4. Credencial válida sin el permiso ⇒ **403**.
//! 5. Con el permiso ⇒ se atiende la petición.
//! 6. **CSRF**: si la credencial es la cookie y el método es mutante
//!    (POST/PATCH/DELETE) se exige `X-CSRF-Token` == `session.csrf_token`; sin
//!    cabecera o incorrecta ⇒ **403** *sin destruir la sesión*. Con Bearer no
//!    se exige CSRF (no es *ambient authority*).
//!
//! La decisión de agrupar el CSRF de API aquí (en lugar del `csrf_guard` global
//! del bloque 2) evita aplicar dos veces la comprobación: [`super::routes`]
//! deja `csrf_guard` solo para `POST /auth/logout`, que es la única mutación de
//! autenticación fuera de la tabla de permisos.

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::state::AppState;

use super::error::AuthError;
use super::session::{SESSION_KEY, SessionState};
use super::{AuthService, csrf, permissions_from_scope};

/// Envuelve `router` exigiendo `permission` en todas sus rutas.
///
/// Usa `route_layer` para que la autorización solo se ejecute en rutas
/// realmente casadas (un 404 de routing no se convierte en 401). Se aplica en
/// [`crate::routes::router`] agrupando cada router de dominio por permiso.
pub fn require_permission(
    router: Router<AppState>,
    state: &AppState,
    permission: &'static str,
) -> Router<AppState> {
    protect(router, state, Some(permission))
}

/// Aplica la capa de autorización con un permiso requerido opcional.
fn protect(
    router: Router<AppState>,
    state: &AppState,
    permission: Option<&'static str>,
) -> Router<AppState> {
    router.route_layer(from_fn_with_state(
        state.clone(),
        move |State(state): State<AppState>,
              session: Session,
              request: Request,
              next: Next| async move {
            authorize(state, permission, session, request, next).await
        },
    ))
}

/// Núcleo de la autorización: elige la vía (Bearer/cookie) y aplica la política.
async fn authorize(
    state: AppState,
    required: Option<&str>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        // Modo abierto: passthrough (no rompe los tests ni el desarrollo).
        return next.run(request).await;
    };

    // Bearer tiene prioridad sobre la cookie y no está sujeto a CSRF.
    if let Some(token) = bearer_token(request.headers()) {
        return match auth.validate_access_token(token).await {
            Ok(claims) => {
                let permissions =
                    permissions_from_scope(claims.scope.as_deref().unwrap_or_default());
                if permission_granted(required, &permissions) {
                    next.run(request).await
                } else {
                    AuthError::Forbidden.into_response()
                }
            }
            Err(_) => AuthError::Unauthorized.into_response(),
        };
    }

    authorize_session(auth, required, session, request, next).await
}

/// Autoriza una petición autenticada por cookie de sesión.
async fn authorize_session(
    auth: &AuthService,
    required: Option<&str>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let mut session_state: SessionState = match session.get(SESSION_KEY).await {
        Ok(Some(state)) => state,
        Ok(None) => return AuthError::Unauthorized.into_response(),
        Err(error) => return AuthError::store(error).into_response(),
    };

    if session_state.auth.is_none() {
        return AuthError::Unauthorized.into_response();
    }

    // Vida absoluta: se mide desde el login y el refresco no la extiende. Se
    // comprueba antes de `ensure_fresh` para que la renovación no la resucite.
    let absolute_ttl = auth.session_config().absolute_ttl;
    if let Some(authenticated) = session_state.auth.as_ref()
        && authenticated.is_past_absolute_ttl(absolute_ttl)
    {
        if session.flush().await.is_err() {
            tracing::warn!("no se pudo destruir la sesión caducada");
        }
        return AuthError::Unauthorized.into_response();
    }

    // CSRF en mutaciones autenticadas por cookie. Un fallo NO destruye la sesión.
    if is_mutating(request.method()) {
        let expected = session_state
            .auth
            .as_ref()
            .map(|auth| auth.csrf_token.as_str())
            .unwrap_or_default();
        let provided = request
            .headers()
            .get("x-csrf-token")
            .and_then(|value| value.to_str().ok());
        if !csrf::verify(expected, provided) {
            return AuthError::Forbidden.into_response();
        }
    }

    let changed = match auth.ensure_fresh(&mut session_state).await {
        // Renovación fallida o usuario ya fuera de los grupos permitidos.
        Err(error) => {
            if session.flush().await.is_err() {
                tracing::warn!("no se pudo destruir la sesión tras un fallo de renovación");
            }
            return error.into_response();
        }
        Ok(changed) => changed,
    };

    let permissions = session_state
        .auth
        .as_ref()
        .map(|auth| auth.permissions.clone())
        .unwrap_or_default();
    if !permission_granted(required, &permissions) {
        return AuthError::Forbidden.into_response();
    }

    // Solo se reescribe la sesión si la renovación la modificó.
    if changed && let Err(error) = session.insert(SESSION_KEY, &session_state).await {
        return AuthError::store(error).into_response();
    }

    next.run(request).await
}

/// Indica si la credencial cumple el permiso exigido.
///
/// `None` exige solo estar autenticado; cualquier credencial válida vale.
fn permission_granted(required: Option<&str>, permissions: &[String]) -> bool {
    match required {
        None => true,
        Some(required) => permissions.iter().any(|permission| permission == required),
    }
}

/// Extrae el token de una cabecera `Authorization: Bearer <token>`.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() {
        Some(token)
    } else {
        None
    }
}

/// Indica si el método HTTP es mutante (sujeto a CSRF por cookie).
fn is_mutating(method: &Method) -> bool {
    matches!(
        method,
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn bearer_token_parses_scheme_case_insensitively() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc"),
        );
        assert_eq!(bearer_token(&headers), Some("abc"));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("bearer  xyz "),
        );
        assert_eq!(bearer_token(&headers), Some("xyz"));
    }

    #[test]
    fn bearer_token_ignores_other_schemes_and_empty() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic abc"));
        assert_eq!(bearer_token(&headers), None);

        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert_eq!(bearer_token(&headers), None);

        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }

    #[test]
    fn permission_granted_checks_required_permission() {
        let permissions = vec!["read:recipes".to_string()];
        assert!(permission_granted(Some("read:recipes"), &permissions));
        assert!(!permission_granted(Some("write:recipes"), &permissions));
        // Sin permiso específico basta con estar autenticado.
        assert!(permission_granted(None, &permissions));
    }

    #[test]
    fn is_mutating_covers_post_patch_delete() {
        assert!(is_mutating(&Method::POST));
        assert!(is_mutating(&Method::PATCH));
        assert!(is_mutating(&Method::DELETE));
        assert!(!is_mutating(&Method::GET));
    }
}
