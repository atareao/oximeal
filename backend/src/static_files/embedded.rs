//! Implementación del servido del frontend con `rust-embed`.

use std::borrow::Cow;

use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, Uri, header};
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
/// 2. Si `mime_guess` reconoce su extensión (`.js`, `.css`, `.png`…), es una
///    petición de fichero: 404 si no existe.
/// 3. Si la extensión es desconocida, solo la tratamos como fichero cuando el
///    nombre es de primer nivel (p. ej. `/foo.unknownext`); los paths anidados
///    con puntos se consideran rutas de cliente.
fn is_orphan_asset(key: &str) -> bool {
    if key.starts_with("assets/") {
        return true;
    }

    let Some(extension) = std::path::Path::new(key)
        .extension()
        .and_then(|ext| ext.to_str())
    else {
        return false;
    };

    if mime_guess::from_ext(extension).first().is_some() {
        return true;
    }

    !key.contains('/')
}

/// Construye la respuesta para un asset existente.
///
/// Recibe el `Cow<'static, [u8]>` de `rust-embed` directamente: `Body` tiene
/// `From<Cow<'static, [u8]>>`, así que no se copia el contenido (zero-copy).
///
/// Los assets bajo `assets/` llevan hash en el nombre y se cachean para
/// siempre; el resto (`index.html`, favicon…) debe revalidarse.
fn asset_response(key: &str, data: Cow<'static, [u8]>) -> Response {
    let content_type = mime_guess::from_path(key).first_or_octet_stream();
    let cache_control = if key.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    let content_type = HeaderValue::from_str(content_type.as_ref())
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

    let mut response = (StatusCode::OK, Body::from(data)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    response
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
/// Solo atiende `GET` y `HEAD`; cualquier otro método responde `405`.
pub async fn serve(method: Method, uri: Uri) -> Response {
    let raw_path = uri.path();

    // La API nunca cae al fallback SPA. Clasificamos sobre la clave normalizada
    // para detectar variantes como `//api/health` o `/./api/health`.
    if is_api_path(raw_path) {
        return api_not_found(raw_path);
    }

    if !matches!(method, Method::GET | Method::HEAD) {
        return AppError::MethodNotAllowed(format!(
            "método {method} no permitido en rutas del frontend"
        ))
        .into_response();
    }

    let key = normalize(raw_path);

    if let Some(content) = Assets::get(&key) {
        return strip_body_for_head(&method, asset_response(&key, content.data));
    }

    // Ruta con pinta de fichero que no existe: asset huérfano, no ruta de cliente.
    if is_orphan_asset(&key) {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Fallback SPA: `index.html` para que lo resuelva el router del cliente.
    match Assets::get("index.html") {
        Some(content) => strip_body_for_head(&method, asset_response("index.html", content.data)),
        None => strip_body_for_head(&method, index_not_packaged()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::static_files::normalize;

    /// Valor de una cabecera como `&str`.
    fn header_str(response: &Response, name: header::HeaderName) -> &str {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
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
        let response = asset_response("assets/index-abc.js", Cow::Borrowed(b"// js"));

        assert_eq!(
            header_str(&response, header::CACHE_CONTROL),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            header_str(&response, header::CONTENT_TYPE),
            "text/javascript"
        );
    }

    #[test]
    fn index_html_is_no_cache_with_html_mime() {
        let response = asset_response("index.html", Cow::Borrowed(b"<html></html>"));

        assert_eq!(header_str(&response, header::CACHE_CONTROL), "no-cache");
        assert_eq!(header_str(&response, header::CONTENT_TYPE), "text/html");
    }

    #[test]
    fn css_asset_gets_css_mime() {
        let response = asset_response("assets/x.css", Cow::Borrowed(b"a{}"));

        assert_eq!(header_str(&response, header::CONTENT_TYPE), "text/css");
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
}
