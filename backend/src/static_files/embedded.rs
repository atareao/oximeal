//! Implementación del servido del frontend con `rust-embed`.

use std::borrow::Cow;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

use super::{api_not_found, is_api_path, normalize};
use crate::error::AppError;

/// Assets del frontend compilado en `frontend/dist`.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../frontend/dist/"]
struct Assets;

/// `true` si la clave debe responderse con 404 (asset huérfano) en vez de
/// caer al *fallback* SPA.
///
/// Una ruta de cliente puede llevar puntos en el último segmento (por ejemplo
/// `/recipe/pasta.v2`), así que "tener extensión" no basta para concluir que se
/// está pidiendo un fichero. La heurística es:
///
/// 1. Si la clave está bajo el prefijo de assets con hash (`assets/`), es un
///    asset: 404 si no existe.
/// 2. Si la extensión tiene un tipo MIME explícito (`.webmanifest`) o
///    `mime_guess` la reconoce (`.js`, `.css`, `.png`…), es una petición de
///    fichero: 404 si no existe.
/// 3. Si la extensión es desconocida, solo la tratamos como fichero cuando el
///    nombre es de primer nivel (p. ej. `/foo.unknownext`); los paths anidados
///    con puntos se consideran rutas de cliente.
fn is_orphan_asset(key: &str) -> bool {
    if key.starts_with("assets/") {
        return true;
    }

    let Some(extension) = extension_of(key) else {
        return false;
    };

    if explicit_mime_for_extension(extension).is_some()
        || mime_guess::from_ext(extension).first().is_some()
    {
        return true;
    }

    !key.contains('/')
}

/// Constante CSP compatible con Ant Design (CSS-in-JS inyecta `style=""`).
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; \
connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; \
form-action 'self'; manifest-src 'self'; worker-src 'self' blob:";

/// `ETag` fuerte `"<hex sha256>"` a partir del hash embebido por `rust-embed`.
fn etag_value(sha256: &[u8; 32]) -> HeaderValue {
    use std::fmt::Write;
    let mut hex = String::with_capacity(66);
    hex.push('"');
    for byte in sha256 {
        let _ = write!(hex, "{byte:02x}");
    }
    hex.push('"');
    HeaderValue::from_str(&hex).expect("hex es ASCII válido")
}

/// Tipos MIME que fijamos explícitamente, con independencia de lo que detecte
/// `mime_guess`.
///
/// El manifiesto web debe servirse como `application/manifest+json` según la
/// especificación de la W3C. Aunque `mime_guess` 2.x ya lo reconoce, fijarlo
/// aquí evita que un cambio de versión de la dependencia degrade el tipo a
/// `application/octet-stream` y rompa la instalación de la PWA.
const EXPLICIT_MIME_TYPES: &[(&str, &str)] = &[("webmanifest", "application/manifest+json")];

/// Tipo MIME explícito para una extensión, si lo tenemos fijado.
fn explicit_mime_for_extension(extension: &str) -> Option<&'static str> {
    EXPLICIT_MIME_TYPES
        .iter()
        .find(|(ext, _)| ext.eq_ignore_ascii_case(extension))
        .map(|(_, mime)| *mime)
}

/// Extensión de la clave, si la tiene.
fn extension_of(key: &str) -> Option<&str> {
    std::path::Path::new(key)
        .extension()
        .and_then(|ext| ext.to_str())
}

/// `Content-Type` del activo, con `; charset=utf-8` para tipos textuales.
fn content_type_for(key: &str) -> HeaderValue {
    if let Some(mime) = extension_of(key).and_then(explicit_mime_for_extension) {
        return HeaderValue::from_static(mime);
    }

    let mime = mime_guess::from_path(key).first_or_octet_stream();
    let value = if mime.type_() == mime_guess::mime::TEXT {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_string()
    };
    HeaderValue::from_str(&value)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))
}

/// `Cache-Control`: `immutable` para assets con hash; `no-cache` para el resto.
fn cache_control_for(key: &str) -> &'static str {
    if key.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

/// Cabeceras de seguridad: `nosniff` siempre; CSP solo en documentos HTML.
fn apply_security_headers(headers: &mut HeaderMap, is_html: bool) {
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if is_html {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(CONTENT_SECURITY_POLICY),
        );
    }
}

/// Añade las cabeceras de seguridad no-HTML (solo `nosniff`) a una respuesta.
fn with_security_headers(mut response: Response) -> Response {
    apply_security_headers(response.headers_mut(), false);
    response
}

/// `true` si `If-None-Match` incluye el `ETag` dado o el comodín `*`.
fn if_none_match_matches(request_headers: &HeaderMap, etag: &HeaderValue) -> bool {
    let Some(value) = request_headers.get(header::IF_NONE_MATCH) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    let etag = etag.to_str().unwrap_or_default();
    value.split(',').any(|token| {
        let token = token.trim();
        token == "*" || token == etag || token.strip_prefix("W/").is_some_and(|t| t == etag)
    })
}

/// Construye la respuesta 200 para un asset existente.
///
/// Recibe el `Cow<'static, [u8]>` de `rust-embed` directamente: `Body` tiene
/// `From<Cow<'static, [u8]>>`, así que no se copia el contenido (zero-copy).
///
/// Los assets bajo `assets/` llevan hash en el nombre y se cachean para
/// siempre; el resto (`index.html`, favicon…) debe revalidarse.
fn asset_response(key: &str, data: Cow<'static, [u8]>, etag: &HeaderValue) -> Response {
    let content_type = content_type_for(key);
    let is_html = content_type.to_str().is_ok_and(|value| {
        value.starts_with("text/html") || value.starts_with("application/xhtml+xml")
    });

    let mut response = (StatusCode::OK, Body::from(data)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control_for(key)),
    );
    headers.insert(header::ETAG, etag.clone());
    apply_security_headers(headers, is_html);
    response
}

/// Responde un asset atendiendo a la revalidación `If-None-Match`.
fn respond(
    method: &Method,
    request_headers: &HeaderMap,
    key: &str,
    file: rust_embed::EmbeddedFile,
) -> Response {
    let sha256 = file.metadata.sha256_hash();
    let etag = etag_value(&sha256);

    if if_none_match_matches(request_headers, &etag) {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        response.headers_mut().insert(header::ETAG, etag);
        return response;
    }

    strip_body_for_head(method, asset_response(key, file.data, &etag))
}

/// Vacía el cuerpo de la respuesta para peticiones `HEAD`.
///
/// Un `fallback` de axum recibe el `HEAD` tal cual (no pasa por el
/// `MethodRouter`, que sí lo procesa), así que hay que vaciar el cuerpo a mano
/// para cumplir con HTTP.
fn strip_body_for_head(method: &Method, mut response: Response) -> Response {
    if *method == Method::HEAD {
        *response.body_mut() = Body::empty();
    }
    response
}

/// 404 con el sobre JSON de la API cuando la feature está activa pero no hay
/// `index.html` empaquetado (build sin `frontend/dist`).
///
/// Se mantiene el formato `{"error": ..., "message": ...}` para no devolver un
/// `404` desnudo sin pistas.
fn index_not_packaged() -> Response {
    AppError::NotFound("UI no empaquetada: ejecuta `just ui-build`".to_string()).into_response()
}

/// Sirve el frontend incrustado con *fallback* SPA.
///
/// Solo atiende `GET` y `HEAD`; cualquier otro método responde `405`. Acepta el
/// `HeaderMap` de la petición para atender la revalidación `If-None-Match`
/// (ETag). La decisión de no soportar `Range` (enviar siempre el recurso
/// completo y no anunciar `Accept-Ranges`) está documentada en `design.md`.
pub async fn serve(method: Method, headers: HeaderMap, uri: Uri) -> Response {
    let raw_path = uri.path();

    // La API nunca cae al fallback SPA. Clasificamos sobre la clave normalizada
    // para detectar variantes como `//api/health` o `/./api/health`.
    if is_api_path(raw_path) {
        return api_not_found(raw_path);
    }

    if !matches!(method, Method::GET | Method::HEAD) {
        return with_security_headers(
            AppError::MethodNotAllowed(format!(
                "método {method} no permitido en rutas del frontend"
            ))
            .into_response(),
        );
    }

    let key = normalize(raw_path);

    if let Some(content) = Assets::get(&key) {
        return respond(&method, &headers, &key, content);
    }

    // Ruta con pinta de fichero que no existe: asset huérfano, no ruta de cliente.
    if is_orphan_asset(&key) {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    }

    // Fallback SPA: `index.html` para que lo resuelva el router del cliente.
    match Assets::get("index.html") {
        Some(content) => respond(&method, &headers, "index.html", content),
        None => strip_body_for_head(&method, with_security_headers(index_not_packaged())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::static_files::normalize;
    use http_body_util::BodyExt;

    /// Valor de una cabecera como `&str`.
    fn header_str(response: &Response, name: header::HeaderName) -> &str {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
    }

    /// Ejecuta `serve` como lo haría el `fallback` del router.
    async fn request(method: Method, path: &str, headers: HeaderMap) -> Response {
        let uri = Uri::try_from(path).expect("URI de test válida");
        serve(method, headers, uri).await
    }

    /// Cuerpo completo de una respuesta como bytes.
    async fn body_bytes(response: Response) -> Vec<u8> {
        response
            .into_body()
            .collect()
            .await
            .expect("cuerpo legible")
            .to_bytes()
            .to_vec()
    }

    // --- normalize ---------------------------------------------------------

    #[test]
    fn normalize_root_is_index() {
        assert_eq!(normalize("/"), "index.html");
        assert_eq!(normalize(""), "index.html");
    }

    #[test]
    fn normalize_keeps_plain_segments() {
        assert_eq!(normalize("/a/b"), "a/b");
    }

    #[test]
    fn normalize_collapses_dots() {
        assert_eq!(normalize("/./x"), "x");
        assert_eq!(normalize("./x"), "x");
        assert_eq!(normalize("/../x"), "x");
    }

    #[test]
    fn normalize_cannot_escape_with_dotdot() {
        assert_eq!(normalize("/../../etc/passwd"), "etc/passwd");
        assert_eq!(normalize("/a/../.."), "index.html");
        assert_eq!(normalize("/../../../etc/passwd"), "etc/passwd");
    }

    #[test]
    fn normalize_leaves_percent_encoding_literal() {
        // No se decodifica: `%2e%2e` nunca se convierte en `..`.
        assert_eq!(normalize("/%2e%2e/x"), "%2e%2e/x");
    }

    #[test]
    fn normalize_treats_backslash_literally() {
        // En Unix `\` no es separador: no puede usarse para escapar del árbol.
        assert_eq!(normalize("/a\\b"), "a\\b");
        assert_eq!(normalize("/..\\..\\etc"), "..\\..\\etc");
    }

    // --- is_orphan_asset ---------------------------------------------------

    #[test]
    fn orphan_asset_known_extension() {
        assert!(is_orphan_asset("assets/index-abc123.js"));
        assert!(is_orphan_asset("assets/index-abc123.css"));
        assert!(is_orphan_asset("favicon.ico"));
    }

    #[test]
    fn orphan_asset_unknown_extension_top_level() {
        assert!(is_orphan_asset("foo.unknownext"));
    }

    #[test]
    fn dotted_client_route_is_not_orphan_asset() {
        assert!(!is_orphan_asset("recipe/pasta.v2"));
    }

    #[test]
    fn assets_directory_without_file_is_not_orphan_asset() {
        // `normalize("/assets/")` produce `assets` (sin extensión): es ruta de cliente.
        assert!(!is_orphan_asset("assets"));
    }

    #[test]
    fn extensionless_route_is_not_orphan_asset() {
        assert!(!is_orphan_asset("shopping"));
        assert!(!is_orphan_asset("menu"));
    }

    // --- asset_response ----------------------------------------------------

    #[test]
    fn hashed_js_asset_is_immutable_with_javascript_mime() {
        let response = asset_response(
            "assets/index-abc.js",
            Cow::Borrowed(b"// js"),
            &HeaderValue::from_static("\"x\""),
        );

        assert_eq!(
            header_str(&response, header::CACHE_CONTROL),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "text/javascript; charset=utf-8"
        );
    }

    #[test]
    fn index_html_is_no_cache_with_html_mime() {
        let response = asset_response(
            "index.html",
            Cow::Borrowed(b"<html></html>"),
            &HeaderValue::from_static("\"x\""),
        );

        assert_eq!(header_str(&response, header::CACHE_CONTROL), "no-cache");
        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn css_asset_gets_css_mime() {
        let response = asset_response(
            "assets/x.css",
            Cow::Borrowed(b"a{}"),
            &HeaderValue::from_static("\"x\""),
        );

        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "text/css; charset=utf-8"
        );
    }

    // --- ETag / cabeceras de seguridad ------------------------------------

    #[test]
    fn etag_value_is_quoted_lowercase_hex() {
        let mut sha = [0u8; 32];
        sha[0] = 0xAB;
        sha[31] = 0x0f;
        let value = etag_value(&sha).to_str().unwrap().to_string();
        assert!(value.starts_with('"') && value.ends_with('"'));
        assert_eq!(value.len(), 66, "32 bytes -> 64 hex + 2 comillas");
        assert!(
            value[1..value.len() - 1]
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert!(value.contains("ab") && value.contains("0f"));
    }

    #[test]
    fn content_type_has_no_charset_for_binary() {
        let value = content_type_for("assets/logo.png")
            .to_str()
            .unwrap()
            .to_string();
        assert!(!value.contains("charset"), "binario sin charset: {value}");
    }

    #[test]
    fn security_headers_nosniff_always_csp_only_html() {
        let mut binary = HeaderMap::new();
        apply_security_headers(&mut binary, false);
        assert_eq!(
            binary.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );
        assert!(binary.get(header::CONTENT_SECURITY_POLICY).is_none());

        let mut html = HeaderMap::new();
        apply_security_headers(&mut html, true);
        assert!(
            html.get(header::CONTENT_SECURITY_POLICY)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("script-src 'self'")
        );
    }

    #[test]
    fn if_none_match_matches_variants() {
        let etag = HeaderValue::from_static("\"abc\"");

        for value in ["\"abc\"", "*", "\"x\", \"abc\"", "W/\"abc\""] {
            let mut headers = HeaderMap::new();
            headers.insert(header::IF_NONE_MATCH, HeaderValue::from_str(value).unwrap());
            assert!(
                if_none_match_matches(&headers, &etag),
                "debería coincidir: {value}"
            );
        }

        let mut stale = HeaderMap::new();
        stale.insert(header::IF_NONE_MATCH, HeaderValue::from_static("\"nope\""));
        assert!(!if_none_match_matches(&stale, &etag));

        assert!(!if_none_match_matches(&HeaderMap::new(), &etag));
    }

    // --- is_api_path (clave normalizada) -----------------------------------

    #[test]
    fn api_detected_on_normalized_key() {
        assert!(is_api_path("/api/health"));
        assert!(is_api_path("/api"));
        assert!(is_api_path("//api/health"));
        assert!(is_api_path("/./api/health"));
        assert!(!is_api_path("/apix/health"));
        assert!(!is_api_path("/"));
    }

    // --- PWA: manifiesto y Service Worker (funciones puras) ---------------

    #[test]
    fn webmanifest_gets_manifest_json_mime() {
        assert_eq!(
            content_type_for("manifest.webmanifest").to_str().unwrap(),
            "application/manifest+json"
        );
    }

    #[test]
    fn webmanifest_mime_is_case_insensitive() {
        assert_eq!(
            content_type_for("manifest.WEBMANIFEST").to_str().unwrap(),
            "application/manifest+json"
        );
    }

    #[test]
    fn explicit_mime_does_not_break_mime_guess_path() {
        // El camino `mime_guess` (y su `; charset=utf-8` para `text/*`) sigue
        // intacto para el resto de extensiones.
        assert_eq!(
            content_type_for("assets/app.js").to_str().unwrap(),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for("assets/app.css").to_str().unwrap(),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn service_worker_is_no_cache_while_hashed_assets_are_immutable() {
        assert_eq!(cache_control_for("sw.js"), "no-cache");
        assert_eq!(
            cache_control_for("assets/x.js"),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn webmanifest_is_treated_as_file_not_client_route() {
        assert!(is_orphan_asset("manifest.webmanifest"));
        assert!(is_orphan_asset("nested/manifest.webmanifest"));
        assert!(is_orphan_asset("assets/missing.js"));
        // Una ruta de cliente con punto sigue siendo ruta de cliente.
        assert!(!is_orphan_asset("recipe/pasta.v2"));
    }

    // --- PWA: manifiesto y Service Worker (end-to-end) --------------------
    //
    // En un clon limpio `build.rs` solo genera un `index.html` placeholder, así
    // que `manifest.webmanifest` y `sw.js` no están embebidos. Estos tests se
    // omiten en ese caso para no depender de `frontend/dist` en CI.

    /// `true` si el asset está realmente embebido (hay build de frontend).
    fn is_embedded(key: &str) -> bool {
        Assets::get(key).is_some()
    }

    #[tokio::test]
    async fn manifest_webmanifest_is_served_with_manifest_mime() {
        if !is_embedded("manifest.webmanifest") {
            return;
        }

        let response = request(Method::GET, "/manifest.webmanifest", HeaderMap::new()).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "application/manifest+json"
        );
    }

    #[tokio::test]
    async fn service_worker_is_served_with_no_cache() {
        if !is_embedded("sw.js") {
            return;
        }

        let response = request(Method::GET, "/sw.js", HeaderMap::new()).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(header_str(&response, header::CACHE_CONTROL), "no-cache");
    }

    #[tokio::test]
    async fn manifest_webmanifest_revalidates_with_etag() {
        if !is_embedded("manifest.webmanifest") {
            return;
        }

        let first = request(Method::GET, "/manifest.webmanifest", HeaderMap::new()).await;
        let etag = first
            .headers()
            .get(header::ETAG)
            .cloned()
            .expect("el manifiesto debe llevar ETag");

        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, etag);
        let second = request(Method::GET, "/manifest.webmanifest", headers).await;

        assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn missing_webmanifest_is_404_and_not_spa_fallback() {
        if !is_embedded("manifest.webmanifest") {
            return;
        }

        let response = request(Method::GET, "/missing.webmanifest", HeaderMap::new()).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(
            !header_str(&response, header::CONTENT_TYPE).starts_with("text/html"),
            "un .webmanifest inexistente no debe servirse como HTML"
        );
        assert!(
            body_bytes(response).await.is_empty(),
            "el 404 de un asset huérfano no debe llevar cuerpo"
        );
    }

    #[tokio::test]
    async fn head_manifest_returns_200_without_body() {
        if !is_embedded("manifest.webmanifest") {
            return;
        }

        let response = request(Method::HEAD, "/manifest.webmanifest", HeaderMap::new()).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "application/manifest+json"
        );
        assert!(
            body_bytes(response).await.is_empty(),
            "HEAD no debe llevar cuerpo"
        );
    }
}
