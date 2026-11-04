//! Autenticación OIDC (BFF): servicio orquestador, sesión server-side y CSRF.
//!
//! El backend actúa como cliente confidencial BFF contra Pocket ID: el
//! navegador nunca ve tokens, solo una cookie opaca `httpOnly`. Este módulo
//! prepara además los tipos que usará la autorización por permisos (bloque 3).

pub mod csrf;
pub mod error;
pub mod jwt;
pub mod middleware;
pub mod oidc;
pub mod routes;
pub mod session;

use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tower_sessions::ExpiredDeletion;
use utoipa::ToSchema;

pub use error::{AuthError, AuthResult};
pub use jwt::AccessTokenClaims;
pub use oidc::OidcClient;
pub use session::{
    AuthenticatedSession, PreAuthFlow, SESSION_KEY, SessionState, SqliteSessionStore,
};

use crate::config::{Config, OidcConfig, SessionConfig};

/// Permisos que el SPA solicita al iniciar sesión (modelo *APIs and permissions*).
pub const SPA_PERMISSIONS: &[&str] = &[
    "read:recipes",
    "write:recipes",
    "read:plan",
    "write:plan",
    "read:shopping",
    "write:shopping",
];

/// Ámbitos OIDC base solicitados en el login.
pub const BASE_SCOPES: &str = "openid profile groups offline_access";

/// Intervalo de la tarea de limpieza de sesiones caducadas.
const DELETION_INTERVAL: Duration = Duration::from_secs(3600);

/// Respuesta de `GET /auth/me`. Nunca incluye tokens.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    /// Indica si la petición va autenticada por cookie de sesión.
    pub authenticated: bool,
    /// Claim `sub` del usuario.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Nombre para mostrar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Permisos concedidos.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    /// Token anti-CSRF para las mutaciones autenticadas por cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
}

impl MeResponse {
    /// Respuesta para una petición sin sesión.
    pub fn anonymous() -> Self {
        Self {
            authenticated: false,
            subject: None,
            name: None,
            permissions: Vec::new(),
            csrf_token: None,
        }
    }

    /// Respuesta para una sesión autenticada.
    pub fn authenticated(auth: &AuthenticatedSession) -> Self {
        Self {
            authenticated: true,
            subject: Some(auth.subject.clone()),
            name: auth.name.clone(),
            permissions: auth.permissions.clone(),
            csrf_token: Some(auth.csrf_token.clone()),
        }
    }
}

/// Servicio de autenticación: orquesta el cliente OIDC y el almacén de sesiones.
#[derive(Debug, Clone)]
pub struct AuthService {
    oidc_config: OidcConfig,
    session_config: SessionConfig,
    oidc: OidcClient,
    store: SqliteSessionStore,
}

impl AuthService {
    /// Construye el servicio desde la configuración y el pool.
    ///
    /// Devuelve `None` si `OIDC_ENABLED=false`. Lanza la tarea de limpieza de
    /// sesiones caducadas si hay un runtime de Tokio activo (en tests y en el
    /// binario siempre lo hay).
    pub fn build(config: &Config, pool: SqlitePool) -> anyhow::Result<Option<Arc<AuthService>>> {
        let Some(oidc_config) = config.oidc.clone() else {
            return Ok(None);
        };

        if config.session.secret.is_none() {
            anyhow::bail!("SESSION_SECRET es obligatorio con OIDC_ENABLED=true");
        }

        let http = reqwest::Client::builder()
            .build()
            .map_err(|error| anyhow::anyhow!("no se pudo crear el cliente HTTP: {error}"))?;

        let store = SqliteSessionStore::new(pool);
        let oidc = OidcClient::new(oidc_config.issuer.clone(), http);

        let service = Arc::new(AuthService {
            oidc_config,
            session_config: config.session.clone(),
            oidc,
            store,
        });

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let store = service.store.clone();
            handle.spawn(async move {
                if let Err(error) = store.continuously_delete_expired(DELETION_INTERVAL).await {
                    tracing::warn!(error = %error, "tarea de limpieza de sesiones terminó");
                }
            });
        }

        Ok(Some(service))
    }

    /// Configuración OIDC.
    pub fn oidc_config(&self) -> &OidcConfig {
        &self.oidc_config
    }

    /// Configuración de sesión.
    pub fn session_config(&self) -> &SessionConfig {
        &self.session_config
    }

    /// Cliente OIDC (discovery/token/JWKS).
    pub fn oidc(&self) -> &OidcClient {
        &self.oidc
    }

    /// Almacén de sesiones, para construir el `SessionManagerLayer`.
    pub fn session_store(&self) -> &SqliteSessionStore {
        &self.store
    }

    /// Indica si al menos un grupo del usuario está en la allowlist.
    pub fn groups_allowed(&self, groups: &[String]) -> bool {
        groups.iter().any(|group| {
            self.oidc_config
                .allowed_groups
                .iter()
                .any(|allowed| allowed == group)
        })
    }

    /// Valida un access token Bearer (RFC 9068) contra el JWKS del proveedor.
    ///
    /// Comprueba algoritmo permitido, firma, `iss`, que `aud` contenga el
    /// *resource* del API y la vigencia `exp`/`nbf`. Devuelve los claims
    /// tipados o [`AuthError::Unauthorized`].
    pub async fn validate_access_token(&self, token: &str) -> AuthResult<AccessTokenClaims> {
        self.oidc
            .validate_access_token(
                token,
                &self.oidc_config.issuer,
                &self.oidc_config.api_audience,
            )
            .await
    }

    /// Renueva el access token si está dentro del margen de expiración.
    ///
    /// Devuelve `Ok(true)` si la sesión se modificó (el llamante debe
    /// persistirla), `Ok(false)` si no había nada que renovar, y
    /// `Err(AuthError::Unauthorized)` si el refresh falla o el usuario ya no
    /// pertenece a ningún grupo permitido; el llamante debe destruir la sesión
    /// en ese caso.
    ///
    /// Tras renovar se **re-derivan los grupos**: preferentemente del
    /// `id_token`; si el proveedor no lo devuelve, del claim `groups` del
    /// access token renovado (Pocket ID lo incluye cuando el scope contiene
    /// `groups`). Si no hay forma de conocerlos, se conservan los previos y se
    /// registra un warning (decisión: no romper sesiones ya establecidas por
    /// una limitación del IdP; el `exp` sigue acotando la vida de la credencial).
    pub async fn ensure_fresh(&self, state: &mut SessionState) -> AuthResult<bool> {
        let Some(auth) = state.auth.as_mut() else {
            return Ok(false);
        };

        let now = chrono::Utc::now().timestamp();
        let margin = self.session_config.refresh_margin.as_secs() as i64;
        if now < auth.access_token_expires_at - margin {
            return Ok(false);
        }

        let Some(refresh_token) = auth.refresh_token.clone() else {
            // Sin refresh token: solo es un fallo si ya ha caducado.
            return if now >= auth.access_token_expires_at {
                Err(AuthError::Unauthorized)
            } else {
                Ok(false)
            };
        };

        let tokens = self
            .oidc
            .refresh(
                &refresh_token,
                &self.oidc_config.client_id,
                &self.oidc_config.client_secret,
            )
            .await
            .map_err(|_| AuthError::Unauthorized)?;

        // 1) El `id_token` es la fuente autoritativa de grupos.
        if let Some(id_token) = tokens.id_token.as_deref() {
            let claims = self
                .oidc
                .verify_id_token(id_token, None, &self.oidc_config.client_id)
                .await?;
            if !self.groups_allowed(&claims.groups) {
                return Err(AuthError::Unauthorized);
            }
            auth.groups = claims.groups;
        } else {
            // 2) Fallback: claim `groups` del access token renovado.
            match self.validate_access_token(&tokens.access_token).await {
                Ok(claims) if !claims.groups.is_empty() => {
                    if !self.groups_allowed(&claims.groups) {
                        return Err(AuthError::Unauthorized);
                    }
                    auth.groups = claims.groups;
                }
                // 3) No podemos conocer los grupos: se conservan los previos.
                _ => {
                    tracing::warn!(
                        "el refresh no trajo id_token ni un access token verificable con grupos; \
                         se conservan los grupos previos de la sesión"
                    );
                }
            }
        }

        auth.permissions = permissions_of(&tokens);
        auth.access_token = tokens.access_token;
        if let Some(refresh_token) = tokens.refresh_token {
            auth.refresh_token = Some(refresh_token);
        }
        if let Some(id_token) = tokens.id_token {
            auth.id_token = Some(id_token);
        }
        auth.access_token_expires_at = now + tokens.expires_in.unwrap_or(3600);

        Ok(true)
    }
}

/// Deriva los permisos del token response, prefiriendo el claim `scope` del
/// access token (RFC 9068) y cayendo al campo `scope` de la respuesta.
pub fn permissions_of(tokens: &oidc::TokenResponse) -> Vec<String> {
    let scope = scope_claim(&tokens.access_token).or_else(|| tokens.scope.clone());
    scope
        .map(|value| permissions_from_scope(&value))
        .unwrap_or_default()
}

/// Divide un `scope` (separado por espacios) en permisos.
pub fn permissions_from_scope(scope: &str) -> Vec<String> {
    scope.split_whitespace().map(str::to_string).collect()
}

/// Lee el claim `scope` del payload de un access token **sin verificar** la firma.
///
/// En el bloque 2 aún no se valida el access token (eso es bloque 3); aquí solo
/// se usa para derivar permisos de forma optimista. Si el token no es un JWT
/// legible, devuelve `None`.
fn scope_claim(access_token: &str) -> Option<String> {
    let payload = access_token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    value
        .get("scope")
        .and_then(|scope| scope.as_str())
        .map(str::to_string)
}

/// Genera un token aleatorio codificado en base64url sin relleno.
pub fn random_token(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    rand::fill(bytes.as_mut_slice());
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Calcula el `code_challenge` PKCE (método `S256`) del `code_verifier`.
pub fn code_challenge_s256(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    let bytes: &[u8] = digest.as_ref();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Acepta solo rutas internas como destino de retorno (evita open redirect).
pub fn sanitize_return_to(value: Option<String>) -> Option<String> {
    let value = value?;
    // Un navegador elimina TAB/CR/LF de la URL al parsearla, así que un valor
    // como "/\t/evil.com" acabaría como "//evil.com", fuera del origen (#80).
    if value.chars().any(char::is_control) {
        return None;
    }
    if value.starts_with('/') && !value.starts_with("//") && !value.contains('\\') {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_tokens_are_unpadded() {
        let token = random_token(32);
        assert!(!token.contains('='));
        assert!(!token.contains('+'));
        assert!(!token.contains('/'));
        assert!(token.len() >= 42);
    }

    #[test]
    fn pkce_challenge_matches_known_vector() {
        // RFC 7636, Apéndice B.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            code_challenge_s256(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn permissions_from_scope_splits_whitespace() {
        assert_eq!(
            permissions_from_scope("openid read:recipes  write:plan"),
            vec!["openid", "read:recipes", "write:plan"]
        );
    }

    #[test]
    fn scope_claim_reads_access_token_payload() {
        // Header/payload sin firma válida: solo importa el payload.
        let payload = URL_SAFE_NO_PAD.encode(br#"{"scope":"read:plan write:plan"}"#);
        let token = format!("header.{payload}.sig");
        assert_eq!(scope_claim(&token).as_deref(), Some("read:plan write:plan"));
    }

    #[test]
    fn sanitize_return_to_rejects_external() {
        assert_eq!(
            sanitize_return_to(Some("/menu".into())),
            Some("/menu".into())
        );
        assert_eq!(sanitize_return_to(Some("//evil.com".into())), None);
        assert_eq!(sanitize_return_to(Some("https://evil.com".into())), None);
        assert_eq!(sanitize_return_to(None), None);
    }
    #[test]
    fn sanitize_return_to_rejects_control_characters() {
        // Un navegador elimina TAB/CR/LF al parsear la URL: "/\t/evil.com" se
        // convierte en "//evil.com" y el 302 acabaría fuera del origen (#80).
        for value in [
            "/\t/evil.com",
            "/\n/evil.com",
            "/\r/evil.com",
            "/\t\t//evil.com",
            "/menu\t",
        ] {
            assert_eq!(
                sanitize_return_to(Some(value.to_string())),
                None,
                "debe rechazarse el carácter de control en {value:?}"
            );
        }
    }

    #[test]
    fn sanitize_return_to_keeps_internal_paths_with_query() {
        for value in [
            "/shopping",
            "/recipes?q=pasta",
            "/recipes?q=pasta&page=2",
            "/recipes?q=a b",
        ] {
            assert_eq!(
                sanitize_return_to(Some(value.to_string())),
                Some(value.to_string()),
                "debe conservarse {value:?}"
            );
        }
    }

    #[test]
    fn sanitize_return_to_output_never_escapes_origin() {
        // Invariante del sink: si un valor se acepta, no puede salir del origen.
        for value in [
            "/shopping",
            "/recipes?q=pasta",
            "/a/b?c=d#e",
            "/\t/evil.com",
            "/\n//evil.com",
        ] {
            if let Some(accepted) = sanitize_return_to(Some(value.to_string())) {
                assert!(accepted.starts_with('/'), "{accepted:?}");
                assert!(!accepted.starts_with("//"), "{accepted:?}");
                assert!(!accepted.chars().any(char::is_control), "{accepted:?}");
            }
        }
    }
}
