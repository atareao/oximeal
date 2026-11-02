//! Cliente OIDC manual: descubrimiento, JWKS con caché, intercambio de código,
//! renovación y validación del ID token.
//!
//! Se implementa a mano (ADR-12) para poder apuntar discovery/token/JWKS a un
//! mock en los tests de integración sin el crate `openidconnect`.

use std::sync::Arc;

use jsonwebtoken::jwk::{Jwk, JwkSet, PublicKeyUse};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Url;
use serde::Deserialize;
use tokio::sync::RwLock;

use super::error::{AuthError, AuthResult};
use super::jwt::AccessTokenClaims;

/// Documento de descubrimiento OIDC (`/.well-known/openid-configuration`).
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryDocument {
    /// Emisor (`iss`).
    pub issuer: String,
    /// Endpoint de autorización.
    pub authorization_endpoint: String,
    /// Endpoint de token.
    pub token_endpoint: String,
    /// URI del JWKS.
    pub jwks_uri: String,
    /// Endpoint de fin de sesión (opcional).
    #[serde(default)]
    pub end_session_endpoint: Option<String>,
}

/// Respuesta del endpoint de token.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    /// Access token.
    pub access_token: String,
    /// Refresh token (si se solicitó `offline_access`).
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// ID token (si se solicitó `openid`).
    #[serde(default)]
    pub id_token: Option<String>,
    /// Tipo de token (habitualmente `Bearer`).
    #[serde(default)]
    pub token_type: Option<String>,
    /// Vida útil del access token en segundos.
    #[serde(default)]
    pub expires_in: Option<i64>,
    /// Ámbitos/permisos concedidos.
    #[serde(default)]
    pub scope: Option<String>,
}

/// Claims relevantes del ID token.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IdTokenClaims {
    /// Sujeto.
    #[serde(default)]
    pub sub: String,
    /// Nombre para mostrar.
    #[serde(default)]
    pub name: Option<String>,
    /// Correo.
    #[serde(default)]
    pub email: Option<String>,
    /// Grupos del usuario.
    #[serde(default)]
    pub groups: Vec<String>,
    /// Nonce anti-replay.
    #[serde(default)]
    pub nonce: Option<String>,
}

/// Cliente OIDC con caché de discovery y JWKS.
#[derive(Debug, Clone)]
pub struct OidcClient {
    http: reqwest::Client,
    issuer: String,
    discovery: Arc<RwLock<Option<DiscoveryDocument>>>,
    jwks: Arc<RwLock<Option<JwkSet>>>,
}

impl OidcClient {
    /// Crea el cliente para el issuer dado.
    pub fn new(issuer: impl Into<String>, http: reqwest::Client) -> Self {
        Self {
            http,
            issuer: issuer.into(),
            discovery: Arc::new(RwLock::new(None)),
            jwks: Arc::new(RwLock::new(None)),
        }
    }

    /// URL del documento de descubrimiento para el issuer configurado.
    fn discovery_url(&self) -> String {
        format!(
            "{}/.well-known/openid-configuration",
            self.issuer.trim_end_matches('/')
        )
    }

    /// Devuelve el documento de descubrimiento, cacheándolo.
    pub async fn discovery(&self) -> AuthResult<DiscoveryDocument> {
        if let Some(document) = self.discovery.read().await.clone() {
            return Ok(document);
        }
        let document = self.fetch_discovery().await?;
        *self.discovery.write().await = Some(document.clone());
        Ok(document)
    }

    /// Fuerza un refresco del documento de descubrimiento.
    async fn fetch_discovery(&self) -> AuthResult<DiscoveryDocument> {
        let response = self
            .http
            .get(self.discovery_url())
            .send()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Upstream(format!(
                "discovery devolvió {}",
                response.status()
            )));
        }

        let document = response
            .json::<DiscoveryDocument>()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))?;

        // Mitigación de *mix-up*: el `issuer` que anuncia el discovery debe
        // coincidir con el configurado (se tolera una barra final de diferencia).
        if document.issuer.trim_end_matches('/') != self.issuer.trim_end_matches('/') {
            return Err(AuthError::Upstream(format!(
                "el issuer del discovery ({}) no coincide con el configurado ({})",
                document.issuer, self.issuer
            )));
        }

        Ok(document)
    }

    /// Devuelve el JWKS, cacheándolo; si falta se descarga.
    async fn jwks(&self) -> AuthResult<JwkSet> {
        if let Some(set) = self.jwks.read().await.clone() {
            return Ok(set);
        }
        self.refresh_jwks().await
    }

    /// Descarga el JWKS y actualiza la caché.
    async fn refresh_jwks(&self) -> AuthResult<JwkSet> {
        let discovery = self.discovery().await?;
        let response = self
            .http
            .get(&discovery.jwks_uri)
            .send()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Upstream(format!(
                "JWKS devolvió {}",
                response.status()
            )));
        }

        let set = response
            .json::<JwkSet>()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))?;
        *self.jwks.write().await = Some(set.clone());
        Ok(set)
    }

    /// Busca la clave de firma adecuada, refrescando el JWKS si no la encuentra.
    ///
    /// Selecciona por `kid` (o la primera válida si el token no trae `kid`) y
    /// filtra las claves que no son de firma (`use != "sig"`) o cuya `alg`
    /// declarada no coincide con el algoritmo esperado.
    async fn signing_key(&self, kid: Option<&str>, algorithm: Algorithm) -> AuthResult<Jwk> {
        let find = |set: &JwkSet| {
            set.keys
                .iter()
                .find(|key| key_matches(key, kid, algorithm))
                .cloned()
        };

        if let Some(key) = find(&self.jwks().await?) {
            return Ok(key);
        }

        // `kid` desconocido: puede ser una rotación de claves. Refresca una vez.
        if let Some(key) = find(&self.refresh_jwks().await?) {
            return Ok(key);
        }

        Err(AuthError::Unauthorized)
    }

    /// Valida un ID token: firma (JWKS), `iss`, `aud`, `exp` y `nonce`.
    ///
    /// El `nonce` solo se comprueba si se pasa `Some`; en la renovación no
    /// aplica.
    pub async fn verify_id_token(
        &self,
        token: &str,
        nonce: Option<&str>,
        client_id: &str,
    ) -> AuthResult<IdTokenClaims> {
        let header = decode_header(token).map_err(|error| {
            tracing::debug!(error = %error, "ID token con cabecera inválida");
            AuthError::Unauthorized
        })?;

        if header.alg != Algorithm::RS256 {
            return Err(AuthError::Unauthorized);
        }

        let jwk = self
            .signing_key(header.kid.as_deref(), Algorithm::RS256)
            .await?;
        let key = DecodingKey::from_jwk(&jwk).map_err(|error| {
            tracing::debug!(error = %error, "JWK no utilizable para verificar");
            AuthError::Unauthorized
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.trim_end_matches('/')]);
        validation.set_audience(&[client_id]);

        let data = decode::<IdTokenClaims>(token, &key, &validation).map_err(|error| {
            tracing::debug!(error = %error, "ID token rechazado");
            AuthError::Unauthorized
        })?;

        if let Some(expected) = nonce
            && data.claims.nonce.as_deref() != Some(expected)
        {
            return Err(AuthError::Unauthorized);
        }

        Ok(data.claims)
    }

    /// Valida un access token (RFC 9068) contra el JWKS, `iss` y `aud` dados.
    ///
    /// Rechaza `alg=none` o cualquier algoritmo fuera de la allowlist. Si el
    /// `kid` no está en el JWKS cacheado, [`Self::signing_key`] refresca el
    /// JWKS una vez antes de fallar.
    pub async fn validate_access_token(
        &self,
        token: &str,
        issuer: &str,
        audience: &str,
    ) -> AuthResult<AccessTokenClaims> {
        let header = decode_header(token).map_err(|error| {
            tracing::debug!(error = %error, "access token con cabecera inválida");
            AuthError::Unauthorized
        })?;

        if !super::jwt::is_allowed_algorithm(header.alg) {
            return Err(AuthError::Unauthorized);
        }

        let jwk = self.signing_key(header.kid.as_deref(), header.alg).await?;
        let key = DecodingKey::from_jwk(&jwk).map_err(|error| {
            tracing::debug!(error = %error, "JWK no utilizable para verificar el access token");
            AuthError::Unauthorized
        })?;

        super::jwt::decode_access_token(token, &key, header.alg, issuer, audience)
    }

    /// Intercambia el `code` por tokens en el endpoint de token (server-side).
    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
        code_verifier: &str,
    ) -> AuthResult<TokenResponse> {
        let discovery = self.discovery().await?;
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code_verifier", code_verifier),
        ];
        self.post_token(&discovery.token_endpoint, &params).await
    }

    /// Renueva el access token a partir del refresh token (`offline_access`).
    pub async fn refresh(
        &self,
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> AuthResult<TokenResponse> {
        let discovery = self.discovery().await?;
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];
        self.post_token(&discovery.token_endpoint, &params).await
    }

    /// Envía una petición `application/x-www-form-urlencoded` al token endpoint.
    async fn post_token(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> AuthResult<TokenResponse> {
        let response = self
            .http
            .post(endpoint)
            .form(params)
            .send()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))?;

        if !response.status().is_success() {
            tracing::debug!(status = %response.status(), "token endpoint rechazó la petición");
            return Err(AuthError::Upstream(format!(
                "token endpoint devolvió {}",
                response.status()
            )));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|error| AuthError::Upstream(error.to_string()))
    }

    /// Construye la URL de autorización con los parámetros dados.
    pub fn build_authorize_url(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> AuthResult<String> {
        let mut url = Url::parse(endpoint).map_err(|error| {
            AuthError::Upstream(format!("authorization_endpoint inválido: {error}"))
        })?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(key, value);
            }
        }
        Ok(url.to_string())
    }
}

/// Indica si una clave del JWKS es apta para verificar un token.
///
/// Filtra por `kid` (si el token lo trae), por `use == "sig"` (solo claves de
/// firma, si el JWKS declara `use`) y por coherencia de `alg` con el algoritmo
/// esperado (si el JWKS declara `alg`). Las claves sin `use`/`alg` se aceptan
/// para no romper proveedores minimalistas.
fn key_matches(key: &Jwk, kid: Option<&str>, algorithm: Algorithm) -> bool {
    if let Some(kid) = kid
        && key.common.key_id.as_deref() != Some(kid)
    {
        return false;
    }

    if let Some(public_key_use) = &key.common.public_key_use
        && *public_key_use != PublicKeyUse::Signature
    {
        return false;
    }

    if let Some(key_algorithm) = key.common.key_algorithm
        && Algorithm::try_from(key_algorithm).ok() != Some(algorithm)
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Deserializa un JWKS de test a partir de JSON.
    fn jwks(value: serde_json::Value) -> JwkSet {
        serde_json::from_value(value).expect("JWKS de test válido")
    }

    #[test]
    fn key_matches_filters_use_and_alg() {
        let set = jwks(json!({
            "keys": [
                // Primera por `kid`, pero es de cifrado: debe descartarse.
                {"kty":"RSA","kid":"k1","use":"enc","alg":"RS256","n":"AQAB","e":"AQAB"},
                // Firma, pero con otra `alg`: también se descarta.
                {"kty":"RSA","kid":"k1","use":"sig","alg":"RS512","n":"AQAB","e":"AQAB"},
                // La correcta.
                {"kty":"RSA","kid":"k1","use":"sig","alg":"RS256","n":"AQAB","e":"AQAB"},
            ]
        }));

        let matched = set
            .keys
            .iter()
            .find(|key| key_matches(key, Some("k1"), Algorithm::RS256))
            .expect("debe encontrar la clave de firma RS256");

        assert_eq!(matched.common.public_key_use, Some(PublicKeyUse::Signature));
        assert_eq!(
            Algorithm::try_from(matched.common.key_algorithm.expect("alg")).ok(),
            Some(Algorithm::RS256)
        );
    }

    #[test]
    fn key_matches_rejects_encryption_only_keys() {
        let set = jwks(json!({
            "keys": [
                {"kty":"RSA","kid":"k1","use":"enc","alg":"RS256","n":"AQAB","e":"AQAB"},
            ]
        }));

        assert!(
            !set.keys
                .iter()
                .any(|key| key_matches(key, Some("k1"), Algorithm::RS256)),
            "una clave de cifrado no debe usarse para verificar firmas"
        );
    }

    #[test]
    fn key_matches_accepts_keys_without_use_or_alg() {
        let set = jwks(json!({
            "keys": [
                {"kty":"RSA","kid":"k1","n":"AQAB","e":"AQAB"},
            ]
        }));

        assert!(
            set.keys
                .iter()
                .any(|key| key_matches(key, Some("k1"), Algorithm::RS256)),
            "una clave sin `use`/`alg` sigue siendo utilizable"
        );
    }

    #[tokio::test]
    async fn discovery_with_mismatched_issuer_is_rejected() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": "https://evil.example/",
                "authorization_endpoint": "https://evil.example/authorize",
                "token_endpoint": "https://evil.example/token",
                "jwks_uri": "https://evil.example/jwks",
            })))
            .mount(&server)
            .await;

        let client = OidcClient::new(server.uri(), reqwest::Client::new());
        let error = client.discovery().await.expect_err("issuer distinto");
        assert!(matches!(error, AuthError::Upstream(_)), "{error:?}");
    }

    #[tokio::test]
    async fn discovery_tolerates_trailing_slash_difference() {
        let server = MockServer::start().await;
        let announced = format!("{}/", server.uri());
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": announced,
                "authorization_endpoint": format!("{}/authorize", server.uri()),
                "token_endpoint": format!("{}/token", server.uri()),
                "jwks_uri": format!("{}/jwks", server.uri()),
            })))
            .mount(&server)
            .await;

        // Configurado sin barra final: debe aceptarse.
        let client = OidcClient::new(server.uri(), reqwest::Client::new());
        let document = client.discovery().await.expect("issuer con barra final");
        assert_eq!(document.issuer, announced);
    }
}
