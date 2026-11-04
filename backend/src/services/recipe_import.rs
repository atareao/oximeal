//! Importación server-side de recetas desde una URL.
//!
//! Descarga el HTML remoto aplicando guardas SSRF y delega el parseo en
//! [`crate::domain::schema_org::extract_recipe`]. La persistencia es idempotente
//! por `source_url`.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use reqwest::Url;
use reqwest::header::{CONTENT_TYPE, LOCATION};
use uuid::Uuid;

use crate::domain::schema_org::extract_recipe;
use crate::error::{AppError, AppResult};
use crate::models::recipe::Recipe;
use crate::repo;
use crate::state::AppState;

/// Máximo de redirecciones HTTP seguidas manualmente antes de abortar.
const MAX_REDIRECTS: usize = 5;
/// Tope de descarga del cuerpo remoto (2 MiB).
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Resultado de una importación.
pub struct ImportOutcome {
    /// Receta creada, o ya existente con la misma `source_url`.
    pub recipe: Recipe,
    /// `true` si se creó en esta llamada; `false` si ya existía (idempotencia).
    pub created: bool,
}

/// Valida la URL contra las guardas SSRF y devuelve la URL normalizada.
///
/// Envoltura de [`check_url`] que descarta la dirección fijada (pin).
pub async fn validate_url(url: &str, allowed_hosts: &[String]) -> AppResult<Url> {
    check_url(url, allowed_hosts).await.map(|(url, _pin)| url)
}

/// Valida la URL y resuelve su host **una sola vez**.
///
/// Comprueba que el esquema sea `http` o `https`, resuelve el host y rechaza
/// rangos privados, loopback, link-local, ULA y `0.0.0.0`/`::` (IPv4 e IPv6).
/// Un host presente en `allowed_hosts` (comparado con el host textual de la URL,
/// sin puerto) omite **únicamente** este veto de rango; el esquema y el resto de
/// guardas siguen aplicándose.
///
/// Devuelve además la [`SocketAddr`] a la que debe conectarse cuando el host es
/// un dominio. Fijar esa dirección evita el **DNS rebinding / TOCTOU**: sin el
/// pin, `reqwest` resolvería de nuevo al conectar y un host malicioso podría
/// devolver una IP pública en la validación y una privada en la conexión. Para
/// hosts que ya son IP literales el pin es `None` (no hay resolución que fijar).
pub async fn check_url(
    url: &str,
    allowed_hosts: &[String],
) -> AppResult<(Url, Option<SocketAddr>)> {
    let parsed =
        Url::parse(url).map_err(|error| AppError::Validation(format!("URL inválida: {error}")))?;

    match parsed.scheme() {
        "http" | "https" => {}
        other => {
            return Err(AppError::Validation(format!(
                "esquema no permitido: {other}"
            )));
        }
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::Validation("la URL no tiene host".to_string()))?;

    // El allowlist se compara con el host textual tal cual aparece en la URL.
    let allowed = allowed_hosts
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(host));

    // IPv6 entre corchetes (`[::1]`) debe quitarse para parsear/resolver.
    let bare_host = host.trim_start_matches('[').trim_end_matches(']');
    let port = parsed.port_or_known_default().unwrap_or(80);

    // Host que ya es IP literal: no hay DNS que fijar.
    if let Ok(ip) = bare_host.parse::<IpAddr>() {
        if !allowed && is_forbidden_ip(&ip) {
            return Err(AppError::Validation(format!(
                "el host {host} apunta a una dirección prohibida"
            )));
        }
        return Ok((parsed, None));
    }

    // Dominio: se resuelve una sola vez, se valida y se fija la dirección de
    // conexión (cierra la ventana entre validación y descarga).
    let addresses = resolve_host(bare_host, port).await?;

    if !allowed
        && addresses
            .iter()
            .any(|address| is_forbidden_ip(&address.ip()))
    {
        return Err(AppError::Validation(format!(
            "el host {host} apunta a una dirección prohibida"
        )));
    }

    Ok((parsed, addresses.into_iter().next()))
}

/// Resuelve un dominio a sus direcciones IP (host:puerto).
async fn resolve_host(host: &str, port: u16) -> AppResult<Vec<SocketAddr>> {
    let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| AppError::BadGateway(format!("no se pudo resolver {host}: {error}")))?
        .collect();

    if resolved.is_empty() {
        return Err(AppError::BadGateway(format!(
            "no se pudo resolver {host}: sin direcciones"
        )));
    }

    Ok(resolved)
}

/// Indica si una IP cae en un rango no enrutable/privado que debe vetarse.
fn is_forbidden_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            v4.is_loopback()          // 127.0.0.0/8
                || v4.is_private()    // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local() // 169.254.0.0/16
                || v4.is_unspecified() // 0.0.0.0
                || v4.is_broadcast()
                || octets[0] == 0 // 0.0.0.0/8 «esta red»
                || (octets[0] == 100 && (octets[1] & 0xc0) == 64) // 100.64.0.0/10 CGNAT
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()              // ::1
                || v6.is_unspecified()    // ::
                || v6.is_unique_local()   // fc00::/7
                || v6.is_unicast_link_local() // fe80::/10
                // IPv4-mapped (`::ffff:127.0.0.1`) y IPv4-compatible (`::127.0.0.1`).
                || v6.to_ipv4_mapped()
                    .or_else(|| v6.to_ipv4())
                    .map(|v4| is_forbidden_ip(&IpAddr::V4(v4)))
                    .unwrap_or(false)
        }
    }
}

/// Descarga la URL, extrae la receta JSON-LD y la persiste de forma idempotente.
///
/// Aplica [`validate_url`] a la URL inicial y a cada redirección, acota el
/// timeout y el tamaño, y exige `Content-Type` HTML. Devuelve un
/// [`ImportOutcome`] que distingue creación (`201`) de reutilización (`200`).
pub async fn import_from_url(state: &AppState, url: &str) -> AppResult<ImportOutcome> {
    let allowed_hosts = &state.config.import_allowed_hosts;
    let (html, final_url) = fetch_html(url, allowed_hosts).await?;

    let mut input = extract_recipe(&html).ok_or_else(|| {
        AppError::UnprocessableEntity("la página no contiene una receta estructurada".to_string())
    })?;

    // La receta extraída pasa por las mismas reglas que una escrita a mano.
    crate::routes::validation::validate_recipe(&input)?;

    // Si el JSON-LD no declara `url`, se usa la URL final como `source_url` para
    // poder deduplicar.
    if input.source_url.is_none() {
        input.source_url = Some(final_url.as_str().to_string());
    }

    let id = Uuid::new_v4().to_string();
    let (recipe, created) =
        repo::recipes::create_or_get_by_source_url(&state.pool, &id, &input).await?;

    Ok(ImportOutcome { recipe, created })
}

/// Descarga el HTML siguiendo redirecciones de forma manual y re-validada.
///
/// Cada salto se resuelve y valida una sola vez (ver [`check_url`]) y se conecta
/// a la IP fijada. Devuelve el cuerpo y la URL final.
async fn fetch_html(start: &str, allowed_hosts: &[String]) -> AppResult<(String, Url)> {
    let mut current = start.to_string();

    for _ in 0..=MAX_REDIRECTS {
        // Resuelve y valida este salto, fijando la IP ya vetada.
        let (url, pin) = check_url(&current, allowed_hosts).await?;
        let client = build_client(&url, pin)?;

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| AppError::BadGateway(format!("fallo de red: {error}")))?;

        let status = response.status();

        if status.is_redirection() {
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    AppError::BadGateway("redirección sin cabecera Location".to_string())
                })?
                .to_string();
            // `join` resuelve `Location` absolutas o relativas.
            let next = url
                .join(&location)
                .map_err(|error| AppError::BadGateway(format!("redirección inválida: {error}")))?;
            // El siguiente salto se revalida (y fija su IP) al inicio del bucle.
            current = next.to_string();
            continue;
        }

        if !status.is_success() {
            return Err(AppError::BadGateway(format!(
                "el origen respondió {status}"
            )));
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if !is_html_content_type(content_type) {
            return Err(AppError::Validation(format!(
                "Content-Type no soportado: {content_type}"
            )));
        }

        let body = read_capped(response, MAX_BODY_BYTES).await?;
        // Una página latin-1 u otra codificación debe poder parsearse igualmente.
        return Ok((String::from_utf8_lossy(&body).into_owned(), url));
    }

    Err(AppError::BadGateway("demasiadas redirecciones".to_string()))
}

/// Construye un cliente que conecta a la IP ya validada del host (pin).
///
/// Fija la resolución DNS del host a la dirección vetada, de modo que `reqwest`
/// no vuelve a resolver al conectar.
fn build_client(url: &Url, pin: Option<SocketAddr>) -> AppResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none());

    if let Some(address) = pin
        && let Some(host) = url.host_str()
    {
        builder = builder.resolve(host, address);
    }

    builder
        .build()
        .map_err(|error| AppError::Internal(anyhow::anyhow!("cliente HTTP: {error}")))
}

/// Indica si un `Content-Type` es HTML servible.
fn is_html_content_type(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value.starts_with("text/html") || value.starts_with("application/xhtml+xml")
}

/// Lee el cuerpo de la respuesta abortando si supera `max` bytes.
async fn read_capped(mut response: reqwest::Response, max: usize) -> AppResult<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| AppError::BadGateway(format!("fallo leyendo la respuesta: {error}")))?
    {
        if body.len() + chunk.len() > max {
            return Err(AppError::Validation(
                "la respuesta supera el tamaño máximo permitido".to_string(),
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_http_schemes() {
        for url in ["file:///etc/passwd", "ftp://example.com/recipe"] {
            let error = validate_url(url, &[])
                .await
                .expect_err("el esquema debe rechazarse");
            assert!(
                matches!(error, AppError::Validation(_)),
                "{url} debería ser 400 de validación, fue: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn rejects_private_and_loopback_hosts_without_allowlist() {
        for url in [
            "http://127.0.0.1/recipe",
            "http://localhost/recipe",
            "http://10.0.0.1/recipe",
            "http://169.254.1.1/recipe",
            "http://[::1]/recipe",
        ] {
            let error = validate_url(url, &[])
                .await
                .expect_err("el rango debe rechazarse");
            assert!(
                matches!(error, AppError::Validation(_)),
                "{url} debería ser 400 de validación, fue: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn allowlisted_host_skips_private_range_veto() {
        for url in [
            "http://127.0.0.1/recipe",
            "http://localhost/recipe",
            "http://10.0.0.1/recipe",
            "http://169.254.1.1/recipe",
            "http://[::1]/recipe",
        ] {
            let host = Url::parse(url)
                .expect("URL de test")
                .host_str()
                .expect("host")
                .to_string();
            let allowed = vec![host];
            assert!(
                validate_url(url, &allowed).await.is_ok(),
                "{url} debería permitirse con el host en el allowlist"
            );
        }
    }

    #[tokio::test]
    async fn rejects_malformed_url() {
        let error = validate_url("no-es-url", &[])
            .await
            .expect_err("URL malformada");
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn rejects_special_ipv4_and_ipv6_ranges() {
        for url in [
            "http://[::ffff:127.0.0.1]/recipe", // IPv4-mapped
            "http://[::127.0.0.1]/recipe",      // IPv4-compatible
            "http://0.0.0.0/recipe",            // «esta red»
            "http://[::]/recipe",               // sin especificar IPv6
            "http://100.64.0.1/recipe",         // CGNAT 100.64.0.0/10
            "http://[fc00::1]/recipe",          // ULA fc00::/7
            "http://[fd00::1]/recipe",          // ULA fd00::/7
            "http://[fe80::1]/recipe",          // link-local fe80::/10
        ] {
            let error = validate_url(url, &[])
                .await
                .expect_err("el rango debe rechazarse");
            assert!(
                matches!(error, AppError::Validation(_)),
                "{url} debería ser 400 de validación, fue: {error:?}"
            );
        }
    }
}
