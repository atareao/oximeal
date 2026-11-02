//! Validación de access tokens JWT (RFC 9068) contra el JWKS del proveedor.
//!
//! El backend acepta como credencial Bearer un access token emitido por Pocket
//! ID. La validación comprueba:
//!
//! - Que el algoritmo de firma esté en una lista permitida; en particular,
//!   `alg=none` y cualquier algoritmo fuera de la lista se rechazan.
//! - La firma contra la clave pública obtenida del JWKS (resuelta por `kid`).
//! - `iss` esperado, que `aud` contenga el *resource* del API y la vigencia
//!   `exp`/`nbf` (con un pequeño margen por desfase de reloj).
//!
//! Los claims derivados ([`AccessTokenClaims`]) dan pie a la autorización por
//! permiso: el `scope` (separado por espacios) es la fuente de permisos en la
//! doble vía cookie/Bearer (ADR-14).

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;

use super::error::{AuthError, AuthResult};

/// Algoritmos de firma admitidos para access tokens.
///
/// Solo asimétricos: nunca HMAC (secreto compartido) ni `none`.
pub const ALLOWED_ALGORITHMS: &[Algorithm] = &[
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::ES256,
    Algorithm::ES384,
];

/// Margen (segundos) aplicado a `exp`/`nbf` por desfase de reloj.
const LEEWAY_SECONDS: u64 = 30;

/// Claims relevantes de un access token RFC 9068.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AccessTokenClaims {
    /// Sujeto (`sub`): usuario o `client_id` del M2M.
    #[serde(default)]
    pub sub: String,
    /// Ámbitos/permisos concedidos, separados por espacios.
    #[serde(default)]
    pub scope: Option<String>,
    /// Grupos del usuario, si el proveedor los incluye en el access token
    /// (Pocket ID lo hace cuando el scope contiene `groups`).
    #[serde(default)]
    pub groups: Vec<String>,
    /// Audiencia (`aud`), que puede ser una cadena o una lista.
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
    /// Instante de caducidad (segundos Unix).
    #[serde(default)]
    pub exp: i64,
}

/// `typ` exigido a un access token RFC 9068.
pub const ACCESS_TOKEN_TYP: &str = "at+jwt";

/// Indica si un algoritmo de firma está permitido.
pub fn is_allowed_algorithm(algorithm: Algorithm) -> bool {
    ALLOWED_ALGORITHMS.contains(&algorithm)
}

/// Indica si el `typ` de la cabecera corresponde a un access token RFC 9068.
///
/// RFC 9068 §2.1 fija `typ=at+jwt` para los access tokens JWT. Se exige, por
/// lo que un token sin `typ` o con otro valor (p. ej. el `JWT` por defecto de
/// un ID token) se rechaza.
pub fn is_access_token_typ(typ: Option<&str>) -> bool {
    typ.is_some_and(|value| value.eq_ignore_ascii_case(ACCESS_TOKEN_TYP))
}

/// Valida un access token ya asociado a su clave de decodificación.
///
/// Comprueba `alg` (debe estar permitido), que la cabecera declare
/// `typ=at+jwt`, `iss`, que `aud` contenga `audience` y la vigencia
/// `exp`/`nbf`. Devuelve los claims tipados o [`AuthError::Unauthorized`] si
/// cualquier comprobación falla.
pub fn decode_access_token(
    token: &str,
    key: &DecodingKey,
    algorithm: Algorithm,
    issuer: &str,
    audience: &str,
) -> AuthResult<AccessTokenClaims> {
    if !is_allowed_algorithm(algorithm) {
        return Err(AuthError::Unauthorized);
    }

    // `decode` valida firma y claims; aquí se comprueba además el `typ`.
    let header = jsonwebtoken::decode_header(token).map_err(|error| {
        tracing::debug!(error = %error, "access token con cabecera inválida");
        AuthError::Unauthorized
    })?;
    if !is_access_token_typ(header.typ.as_deref()) {
        tracing::debug!(typ = ?header.typ, "access token con `typ` incorrecto");
        return Err(AuthError::Unauthorized);
    }

    let mut validation = Validation::new(algorithm);
    // Se acepta el issuer configurado con o sin barra final.
    let issuer_trimmed = issuer.trim_end_matches('/');
    validation.set_issuer(&[issuer, issuer_trimmed]);
    validation.set_audience(&[audience]);
    validation.leeway = LEEWAY_SECONDS;
    // `nbf` solo se valida si el token lo incluye.
    validation.validate_nbf = true;

    decode::<AccessTokenClaims>(token, key, &validation)
        .map(|data| data.claims)
        .map_err(|error| {
            tracing::debug!(error = %error, "access token rechazado");
            AuthError::Unauthorized
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;

    const ISSUER: &str = "https://idp.example/";
    const AUDIENCE: &str = "https://oximeal.local/api";

    /// Par RSA estático de test (solo para unit tests, nunca en producción).
    const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
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

    /// Clave pública correspondiente.
    const PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0NENL8YizQbUgEqV/dBg
h3yqiY+3Az0efDHotSuiZwxowvzbJTIse0s7Mnc2mQCJDyRlN4AjlcKYbD0KD8Zd
68//2Zro14wD5nP1nLOv5u5gFs5t9wOOP0WbOdA203hQV36bjRsFQoTSCnxazBZb
1raXShMK/7E2Vb7xGGGKwOKhAE0QTDqmO6R8QCE5N/6d6ESO3PYD8F+jllY+08MJ
P8mF5AWU+UijFIx95Rt25CYsaaOsXOcbLHCVv+zS5KOJvXTYAxhJaQqBT14iVpx1
BPwnSod4+W5vdfaL/ksdCUrfx4ow6PmiQ41zkIYJLNXWyOXuHl1EzYc0NwP9NdeI
xwIDAQAB
-----END PUBLIC KEY-----
"#;

    #[derive(Serialize)]
    struct Claims<'a> {
        sub: &'a str,
        scope: &'a str,
        iss: &'a str,
        aud: &'a str,
        exp: i64,
        nbf: i64,
    }

    fn token(issuer: &str, audience: &str, scope: &str, exp: i64, nbf: i64) -> String {
        token_with_typ(issuer, audience, scope, exp, nbf, Some(ACCESS_TOKEN_TYP))
    }

    /// Igual que [`token`] pero con un `typ` de cabecera explícito.
    fn token_with_typ(
        issuer: &str,
        audience: &str,
        scope: &str,
        exp: i64,
        nbf: i64,
        typ: Option<&str>,
    ) -> String {
        let claims = Claims {
            sub: "user-1",
            scope,
            iss: issuer,
            aud: audience,
            exp,
            nbf,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test".to_string());
        header.typ = typ.map(str::to_string);
        let key = EncodingKey::from_rsa_pem(PRIVATE_KEY_PEM.as_bytes()).expect("clave RSA");
        encode(&header, &claims, &key).expect("firmar")
    }

    fn decoding_key() -> DecodingKey {
        DecodingKey::from_rsa_pem(PUBLIC_KEY_PEM.as_bytes()).expect("clave pública")
    }

    #[test]
    fn allowed_algorithms_exclude_none_and_hmac() {
        assert!(is_allowed_algorithm(Algorithm::RS256));
        assert!(is_allowed_algorithm(Algorithm::ES384));
        assert!(!is_allowed_algorithm(Algorithm::HS256));
    }

    #[test]
    fn valid_token_yields_claims() {
        let now = chrono::Utc::now().timestamp();
        let token = token(
            ISSUER,
            AUDIENCE,
            "read:recipes write:recipes",
            now + 600,
            now,
        );
        let claims =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect("token válido");
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.scope.as_deref(), Some("read:recipes write:recipes"));
    }

    #[test]
    fn expired_token_is_rejected() {
        let now = chrono::Utc::now().timestamp();
        let token = token(ISSUER, AUDIENCE, "read:recipes", now - 600, now - 1200);
        let error =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect_err("caducado");
        assert!(matches!(error, AuthError::Unauthorized));
    }

    #[test]
    fn foreign_audience_is_rejected() {
        let now = chrono::Utc::now().timestamp();
        let token = token(
            ISSUER,
            "https://other.example/api",
            "read:recipes",
            now + 600,
            now,
        );
        let error =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect_err("aud ajeno");
        assert!(matches!(error, AuthError::Unauthorized));
    }

    #[test]
    fn wrong_issuer_is_rejected() {
        let now = chrono::Utc::now().timestamp();
        let token = token(
            "https://evil.example/",
            AUDIENCE,
            "read:recipes",
            now + 600,
            now,
        );
        let error =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect_err("iss ajeno");
        assert!(matches!(error, AuthError::Unauthorized));
    }

    #[test]
    fn access_token_typ_is_required() {
        let now = chrono::Utc::now().timestamp();
        for typ in [Some("JWT"), Some("id+jwt"), None] {
            let token = token_with_typ(ISSUER, AUDIENCE, "read:recipes", now + 600, now, typ);
            let error =
                decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                    .expect_err("typ de access token no válido");
            assert!(matches!(error, AuthError::Unauthorized), "typ={typ:?}");
        }
    }

    #[test]
    fn access_token_typ_is_case_insensitive() {
        let now = chrono::Utc::now().timestamp();
        let token = token_with_typ(
            ISSUER,
            AUDIENCE,
            "read:recipes",
            now + 600,
            now,
            Some("AT+JWT"),
        );
        let claims =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect("typ AT+JWT debe aceptarse");
        assert_eq!(claims.sub, "user-1");
    }

    #[test]
    fn nbf_in_the_future_is_rejected() {
        let now = chrono::Utc::now().timestamp();
        let token = token(ISSUER, AUDIENCE, "read:recipes", now + 7200, now + 3600);
        let error =
            decode_access_token(&token, &decoding_key(), Algorithm::RS256, ISSUER, AUDIENCE)
                .expect_err("nbf futuro");
        assert!(matches!(error, AuthError::Unauthorized));
    }
}
