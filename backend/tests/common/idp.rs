//! Mock de proveedor de identidad OIDC para los tests de integración.
//!
//! Sirve discovery, JWKS y token endpoint con `wiremock`. El ID token se firma
//! con una clave RSA estática de test (nunca usada en producción).

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// `kid` de la clave de firma de test (debe coincidir entre JWKS y cabecera JWT).
pub const KID: &str = "oximeal-test-key-1";

/// `client_id` esperado en el claim `aud` del ID token.
pub const CLIENT_ID: &str = "oximeal-spa";

/// Resource del API (`aud` del access token) configurado en el estado de test.
pub const API_AUDIENCE: &str = "https://oximeal.local/api";

/// Grupos que emite el ID token del mock.
pub const USER_GROUPS: &[&str] = &["oximeal-users"];

/// Refresh token que el token endpoint rechaza (para el escenario de fallo).
pub const INVALID_REFRESH: &str = "invalid-refresh";

/// Refresh cuyo `id_token` renueva pero con un grupo NO permitido (401).
pub const NO_GROUPS_REFRESH: &str = "refresh-no-groups";

/// Refresh que devuelve grupos solo en el access token (sin `id_token`).
pub const ACCESS_GROUPS_REFRESH: &str = "refresh-access-groups";

/// Refresh cuyo access token trae un grupo NO permitido (sin `id_token`).
pub const ACCESS_NO_GROUPS_REFRESH: &str = "refresh-access-no-groups";

/// Refresh que no devuelve ni `id_token` ni un access token verificable.
pub const NO_ID_TOKEN_REFRESH: &str = "refresh-no-id-token";

/// Variante del ID token que emite el token endpoint (tests de validación).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IdTokenVariant {
    /// Token correcto (comportamiento por defecto).
    #[default]
    Normal,
    /// `nbf` en el futuro: el token todavía no es válido.
    NbfInFuture,
    /// `azp` distinto del `client_id` esperado.
    ForeignAzp,
}

/// Clave privada RSA estática de test (solo tests).
pub const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDQ0Q0vxiLNBtSA
SpX90GCHfKqJj7cDPR58Mei1K6JnDGjC/NslMix7SzsydzaZAIkPJGU3gCOVwphs
PQoPxl3rz//ZmujXjAPmc/Wcs6/m7mAWzm33A44/RZs50DbTeFBXfpuNGwVChNIK
fFrMFlvWtpdKEwr/sTZVvvEYYYrA4qEATRBMOqY7pHxAITk3/p3oRI7c9gPwX6OW
Vj7Twwk/yYXkBZT5SKMUjH3lG3bkJixpo6xc5xsscJW/7NLko4m9dNgDGElpCoFP
XiJWnHUE/CdKh3j5bm919ov+Sx0JSt/HijDo+aJDjXOQhgks1dbI5e4eXUTNhzQ3
A/0114jHAgMBAAECggEANlITnJht6Cj9Max1bVP+MHCpSmS05qZJHmS7VRnXQvhS
7lppxbjSIna1HREvBklB3ksI+5GdcDFxhkKwsd7YAuaWd2yZYq0dPJ9wxsVlnmzn
WTd1Grt1jxcT5Mu533QebkxUubXMPfwgSbMEya+NT4MrQw1GILLgYBjuTfnd5wP0
7HmYLp3cvy3MJcLZp92tpScTheyceNHjPzF/MJd5q/0a5fjXi3HAGUrh0OExFESa
QTniccyv3ojWCD2MhhlgD5BHFkghuNHe1KHDJKIB63Us8GXr1A5k+fxhd/gN2frg
JnMOWKHPhSOOAMzm0v2V8MlJBDNS6V/95tj10CnvuQKBgQD3hzu6hhvevBBS+D1+
n/k3eglH+mP/ytNIe0k25s1bXT1XN2U2wXJ9rcLUo6g68QQTl6Rh203xd2xslqtE
uHGOiyKOUAROM6OyZ70J9EchU4C3JgdVNwCNPn4vOcw5RaheJvhFyeutS2FjRnrq
lc7z4B9dxaM2hWO3xjsK/lVCTwKBgQDX9qN7t1QtnSrGMICQh+OPuiJPaBcqG61n
sgKr846F4gZ/KKmQqPZmWFHzvQ/E1/OXlI6nUTjqWfD0T65GTc1yYwkkjJD3fizY
IdCYHJTw/Hpjc/WwXKuJ7pecf2Y2sIBCrOMz+d1149EcFDZqjMej2+0TR1c2Hb/8
bJj5pduMCQKBgQC9uPGW7eO+vPbmIU9BFuy2ENu8R7cOADAhhh1/7zfHp5e0mckC
RxOyMNT8ltQDGWi+e6SLPMRpJ9f7AyES8yxWzV4XJk2mgv4daFYEoy9B4/bsakGE
lzvdBuFA4VQfMbzzLXtuQm5ZzUVACm18EIcndcmU3j9TMrpTyaTHsFk5EQKBgAGM
fUOn3uys0d+1/VYMA2/Gb/SN1Iyywh2aQzakeoMvScijrAOjkpDNVVkgI/rPaanO
oO9GP9g+aOH3qFJMwIpvd2NRtsBvoTazl2Ff4xmPVflcnYh0Qa6tjVzL9RmzTlgs
ET0QL6xmw9slsfY1qEDelOasfk2fDP9sb/RqYhaxAoGBAIPnyX3iQIpPLaPeCClE
9RLYCPAB0UJ6lrFaKnhHX+lOmB28ESDId8OhkSIbBZd7Ru1yF9rmuW3Y6AVChu2e
VPsozwVhUdrvOMgdQ2nOJqd7Z0A/9Pb1pMq0OFyqQv6kmx9+2iGGrK8i4G50Pp89
6TbC8J/q0aQEV0T87lpcduwU
-----END PRIVATE KEY-----
"#;

/// Módulo (base64url) de la clave pública JWK de test.
const JWK_MODULUS: &str = "0NENL8YizQbUgEqV_dBgh3yqiY-3Az0efDHotSuiZwxowvzbJTIse0s7Mnc2mQCJDyRlN4AjlcKYbD0KD8Zd68__2Zro14wD5nP1nLOv5u5gFs5t9wOOP0WbOdA203hQV36bjRsFQoTSCnxazBZb1raXShMK_7E2Vb7xGGGKwOKhAE0QTDqmO6R8QCE5N_6d6ESO3PYD8F-jllY-08MJP8mF5AWU-UijFIx95Rt25CYsaaOsXOcbLHCVv-zS5KOJvXTYAxhJaQqBT14iVpx1BPwnSod4-W5vdfaL_ksdCUrfx4ow6PmiQ41zkIYJLNXWyOXuHl1EzYc0NwP9NdeIxw";

/// Proveedor de identidad simulado.
pub struct MockIdp {
    /// Servidor `wiremock`.
    pub server: MockServer,
    /// Issuer (`iss`) del proveedor.
    pub issuer: String,
    /// Endpoint de autorización.
    pub authorization_endpoint: String,
    /// Endpoint de token.
    pub token_endpoint: String,
    /// URI del JWKS.
    pub jwks_uri: String,
    /// Endpoint de fin de sesión (`end_session_endpoint`), si se publica.
    pub end_session_endpoint: Option<String>,
}

impl MockIdp {
    /// Arranca el mock con ID tokens correctos.
    pub async fn start() -> Self {
        Self::start_with(IdTokenVariant::Normal).await
    }

    /// Arranca el mock emitiendo la variante de ID token indicada.
    pub async fn start_with(variant: IdTokenVariant) -> Self {
        Self::start_with_options(variant, true, false).await
    }

    /// Arranca el mock sin publicar `end_session_endpoint` en el discovery.
    pub async fn start_without_end_session() -> Self {
        Self::start_with_options(IdTokenVariant::Normal, false, false).await
    }

    /// Arranca el mock sirviendo primero un JWKS **vacío** y, a partir de la
    /// segunda petición, el JWKS con la clave real.
    ///
    /// Sirve para ejercitar el refresco de JWKS ante un `kid` desconocido: el
    /// primer `signing_key` no encuentra la clave en el JWKS cacheado (vacío) y
    /// fuerza un refresco que sí la trae.
    pub async fn start_with_rotating_jwks() -> Self {
        Self::start_with_options(IdTokenVariant::Normal, true, true).await
    }

    /// Montaje único del mock: discovery, JWKS (estático o rotatorio) y token.
    async fn start_with_options(
        variant: IdTokenVariant,
        with_end_session: bool,
        rotating_jwks: bool,
    ) -> Self {
        let server = MockServer::start().await;
        let issuer = server.uri();
        let authorization_endpoint = format!("{issuer}/authorize");
        let token_endpoint = format!("{issuer}/token");
        let jwks_uri = format!("{issuer}/jwks");
        let end_session_endpoint = with_end_session.then(|| format!("{issuer}/logout"));

        let mut discovery = json!({
            "issuer": issuer,
            "authorization_endpoint": authorization_endpoint,
            "token_endpoint": token_endpoint,
            "jwks_uri": jwks_uri,
        });
        if let Some(endpoint) = &end_session_endpoint {
            discovery["end_session_endpoint"] = json!(endpoint);
        }

        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(discovery))
            .mount(&server)
            .await;

        if rotating_jwks {
            let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            Mock::given(method("GET"))
                .and(path("/jwks"))
                .respond_with(move |_request: &Request| {
                    let call = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if call == 0 {
                        ResponseTemplate::new(200).set_body_json(json!({ "keys": [] }))
                    } else {
                        ResponseTemplate::new(200).set_body_json(test_jwks())
                    }
                })
                .mount(&server)
                .await;
        } else {
            Mock::given(method("GET"))
                .and(path("/jwks"))
                .respond_with(ResponseTemplate::new(200).set_body_json(test_jwks()))
                .mount(&server)
                .await;
        }

        let token_issuer = issuer.clone();
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(move |request: &Request| token_response(request, &token_issuer, variant))
            .mount(&server)
            .await;

        Self {
            server,
            issuer,
            authorization_endpoint,
            token_endpoint,
            jwks_uri,
            end_session_endpoint,
        }
    }
}

/// JWKS de test con la clave pública RSA estática.
fn test_jwks() -> serde_json::Value {
    json!({
        "keys": [{
            "kty": "RSA",
            "kid": KID,
            "use": "sig",
            "alg": "RS256",
            "n": JWK_MODULUS,
            "e": "AQAB",
        }]
    })
}

/// Responde al token endpoint según el `grant_type`.
fn token_response(request: &Request, issuer: &str, variant: IdTokenVariant) -> ResponseTemplate {
    let params = parse_form(&request.body);
    let grant = params
        .get("grant_type")
        .map(String::as_str)
        .unwrap_or_default();

    if grant == "refresh_token" {
        let refresh_token = params
            .get("refresh_token")
            .map(String::as_str)
            .unwrap_or_default();
        if refresh_token == INVALID_REFRESH {
            return ResponseTemplate::new(400).set_body_json(json!({ "error": "invalid_grant" }));
        }

        // Renovación que devuelve los grupos en el `id_token` (camino normal),
        // salvo los casos que los exponen en el access token o no los traen.
        let mut body = json!({
            "access_token": fake_access_token("openid read:recipes write:recipes"),
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "openid read:recipes write:recipes",
            "refresh_token": refresh_token,
        });

        match refresh_token {
            NO_GROUPS_REFRESH => {
                body["id_token"] = json!(sign_id_token_with(issuer, "", &["other-group"], variant));
            }
            ACCESS_GROUPS_REFRESH => {
                body["access_token"] = json!(signed_access_token(
                    issuer,
                    "openid read:recipes",
                    USER_GROUPS
                ));
            }
            ACCESS_NO_GROUPS_REFRESH => {
                body["access_token"] = json!(signed_access_token(
                    issuer,
                    "openid read:recipes",
                    &["other-group"]
                ));
            }
            // Sin `id_token` ni access token verificable: grupos desconocidos.
            NO_ID_TOKEN_REFRESH => {}
            _ => {
                body["id_token"] = json!(sign_id_token_with(issuer, "", USER_GROUPS, variant));
            }
        }

        return ResponseTemplate::new(200).set_body_json(body);
    }

    // authorization_code: el `code` transporta el nonce que debe llevar el ID token.
    let nonce = params.get("code").cloned().unwrap_or_default();
    let id_token = sign_id_token_with(issuer, &nonce, USER_GROUPS, variant);

    ResponseTemplate::new(200).set_body_json(json!({
        "access_token": fake_access_token("openid read:recipes write:recipes"),
        "token_type": "Bearer",
        "expires_in": 3600,
        "scope": "openid read:recipes write:recipes",
        "refresh_token": "refresh-ok",
        "id_token": id_token,
    }))
}

/// Firma un access token RS256 real (verificable) con el claim `groups`.
fn signed_access_token(issuer: &str, scope: &str, groups: &[&str]) -> String {
    sign_access_token_with_groups(issuer, scope, API_AUDIENCE, KID, 3600, 0, groups)
}

/// Firma un ID token RS256 con la clave de test y los grupos dados.
fn sign_id_token(issuer: &str, nonce: &str, groups: &[&str]) -> String {
    sign_id_token_with(issuer, nonce, groups, IdTokenVariant::Normal)
}

/// Firma un ID token RS256 con la variante indicada.
fn sign_id_token_with(
    issuer: &str,
    nonce: &str,
    groups: &[&str],
    variant: IdTokenVariant,
) -> String {
    #[derive(Serialize)]
    struct Claims<'a> {
        sub: &'a str,
        name: &'a str,
        email: &'a str,
        groups: &'a [&'a str],
        nonce: &'a str,
        iss: &'a str,
        aud: &'a str,
        exp: i64,
        iat: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nbf: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        azp: Option<&'a str>,
    }

    let now = chrono::Utc::now().timestamp();
    let (nbf, azp) = match variant {
        IdTokenVariant::Normal => (None, None),
        IdTokenVariant::NbfInFuture => (Some(now + 3600), None),
        IdTokenVariant::ForeignAzp => (None, Some("other-client")),
    };
    let claims = Claims {
        sub: "user-123",
        name: "Test User",
        email: "test@example.com",
        groups,
        nonce,
        iss: issuer,
        aud: CLIENT_ID,
        exp: now + 3600,
        iat: now,
        nbf,
        azp,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.to_string());

    let key = EncodingKey::from_rsa_pem(PRIVATE_KEY_PEM.as_bytes()).expect("clave RSA de test");
    encode(&header, &claims, &key).expect("firmar ID token")
}

/// Claims de un access token RFC 9068 de test.
#[derive(Serialize)]
struct AccessClaims<'a> {
    sub: &'a str,
    scope: &'a str,
    #[serde(skip_serializing_if = "slice_is_empty")]
    groups: &'a [&'a str],
    iss: &'a str,
    aud: &'a str,
    exp: i64,
    iat: i64,
    nbf: i64,
}

/// Omite el claim `groups` cuando está vacío.
fn slice_is_empty(slice: &&[&str]) -> bool {
    slice.is_empty()
}

/// Mintea un access token RS256 válido, con `kid` conocido, `iss` y `aud` dados.
///
/// El `scope` se emite tal cual (permisos separados por espacios).
pub fn access_token(issuer: &str, scope: &str, audience: &str) -> String {
    sign_access_token(issuer, scope, audience, KID, 3600, 0)
}

/// Mintea un access token con un `kid` concreto (para probar rotación de claves).
pub fn access_token_with_kid(issuer: &str, scope: &str, audience: &str, kid: &str) -> String {
    sign_access_token(issuer, scope, audience, kid, 3600, 0)
}

/// Mintea un access token ya caducado (`exp` en el pasado).
pub fn expired_access_token(issuer: &str, scope: &str, audience: &str) -> String {
    sign_access_token(issuer, scope, audience, KID, -3600, -7200)
}

/// Mintea un access token con `nbf` en el futuro (aún no válido).
pub fn nbf_in_future_access_token(issuer: &str, scope: &str, audience: &str) -> String {
    sign_access_token(issuer, scope, audience, KID, 3600, 3600)
}

/// Mintea un token con `typ` de cabecera distinto de `at+jwt`.
pub fn access_token_with_typ(
    issuer: &str,
    scope: &str,
    audience: &str,
    typ: Option<&str>,
) -> String {
    sign_access_token_full(AccessTokenSpec {
        issuer,
        scope,
        audience,
        kid: KID,
        groups: &[],
        exp_offset: 3600,
        nbf_offset: 0,
        typ,
    })
}

/// Mintea un token con `alg=none` (JWT sin firma). Codificado a mano porque
/// `jsonwebtoken` no permite emitir `none`.
pub fn alg_none_token(issuer: &str, scope: &str, audience: &str) -> String {
    let now = chrono::Utc::now().timestamp();
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"at+jwt"}"#);
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({
            "sub": "user-123",
            "scope": scope,
            "iss": issuer,
            "aud": audience,
            "exp": now + 3600,
            "iat": now,
            "nbf": now,
        }))
        .expect("serializar payload alg=none"),
    );
    format!("{header}.{payload}.")
}

/// Firma un access token con la clave RSA de test.
fn sign_access_token(
    issuer: &str,
    scope: &str,
    audience: &str,
    kid: &str,
    exp_offset: i64,
    nbf_offset: i64,
) -> String {
    sign_access_token_full(AccessTokenSpec {
        issuer,
        scope,
        audience,
        kid,
        groups: &[],
        exp_offset,
        nbf_offset,
        typ: Some("at+jwt"),
    })
}

/// Parámetros de un access token de test.
struct AccessTokenSpec<'a> {
    issuer: &'a str,
    scope: &'a str,
    audience: &'a str,
    kid: &'a str,
    groups: &'a [&'a str],
    exp_offset: i64,
    nbf_offset: i64,
    typ: Option<&'a str>,
}

/// Firma un access token con el `typ` de cabecera indicado (o sin `typ`).
fn sign_access_token_full(spec: AccessTokenSpec<'_>) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = AccessClaims {
        sub: "user-123",
        scope: spec.scope,
        groups: spec.groups,
        iss: spec.issuer,
        aud: spec.audience,
        exp: now + spec.exp_offset,
        iat: now + spec.nbf_offset,
        nbf: now + spec.nbf_offset,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(spec.kid.to_string());
    header.typ = spec.typ.map(str::to_string);

    let key = EncodingKey::from_rsa_pem(PRIVATE_KEY_PEM.as_bytes()).expect("clave RSA de test");
    encode(&header, &claims, &key).expect("firmar access token")
}

/// Firma un access token RS256 real (verificable) con grupos.
fn sign_access_token_with_groups(
    issuer: &str,
    scope: &str,
    audience: &str,
    kid: &str,
    exp_offset: i64,
    nbf_offset: i64,
    groups: &[&str],
) -> String {
    sign_access_token_full(AccessTokenSpec {
        issuer,
        scope,
        audience,
        kid,
        groups,
        exp_offset,
        nbf_offset,
        typ: Some("at+jwt"),
    })
}

/// Access token ficticio (JWT sin firmar) que transporta el claim `scope`.
fn fake_access_token(scope: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"at+jwt"}"#);
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({ "scope": scope }))
            .expect("serializar payload del access token"),
    );
    format!("{header}.{payload}.signature")
}

/// Parsea un cuerpo `application/x-www-form-urlencoded`.
fn parse_form(body: &[u8]) -> std::collections::HashMap<String, String> {
    let text = String::from_utf8_lossy(body);
    text.split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            Some((decode_component(key), decode_component(value)))
        })
        .collect()
}

/// Decodifica `application/x-www-form-urlencoded` (`+` → espacio, `%XX`).
fn decode_component(value: &str) -> String {
    let replaced = value.replace('+', " ");
    let bytes = replaced.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&replaced[index + 1..index + 3], 16)
        {
            out.push(byte);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
