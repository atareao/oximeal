//! Tests de integración del servido del frontend y del *fallback* SPA.

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, Request, StatusCode, header};
use http_body_util::BodyExt;
use oximeal_backend::app;
use oximeal_backend::state::AppState;
use tower::ServiceExt;

/// Respuesta cruda de una petición `GET`.
struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

/// Ejecuta un `GET` contra el router completo.
async fn get(state: AppState, uri: &str) -> Reply {
    request(state, "GET", uri).await
}

/// Ejecuta una petición con un método arbitrario contra el router completo.
async fn request(state: AppState, method: &str, uri: &str) -> Reply {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("construir request");

    let response = app(state).oneshot(request).await.expect("ejecutar request");

    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes()
        .to_vec();

    Reply {
        status,
        headers,
        body,
    }
}

/// Devuelve el valor de una cabecera como `&str` (vacío si falta o no es ASCII).
fn header_str(headers: &HeaderMap, name: HeaderName) -> &str {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

/// Cuerpo como texto en minúsculas, para comparaciones laxas de HTML.
fn body_lower(reply: &Reply) -> String {
    String::from_utf8_lossy(&reply.body).to_lowercase()
}

// ---------------------------------------------------------------------------
// Comportamiento común a ambos modos
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_health_is_not_captured_by_fallback() {
    let reply = get(common::test_state().await, "/api/health").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );

    let json: serde_json::Value = serde_json::from_slice(&reply.body).expect("parsear JSON");
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn unknown_api_route_returns_json_404_never_index() {
    let reply = get(common::test_state().await, "/api/no-existe").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );

    let text = body_lower(&reply);
    assert!(
        !text.contains("<html") && !text.contains("<!doctype"),
        "el fallback SPA no debe capturar /api/*: {text}"
    );

    let json: serde_json::Value = serde_json::from_slice(&reply.body).expect("parsear JSON");
    assert_eq!(json["error"], "not_found");
}

#[tokio::test]
async fn missing_asset_returns_404_without_spa_fallback() {
    let reply = get(common::test_state().await, "/assets/index-abc123.js").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let text = body_lower(&reply);
    assert!(
        !text.contains("<html") && !text.contains("<!doctype"),
        "un asset inexistente no debe devolver index.html: {text}"
    );
}

#[tokio::test]
async fn api_exact_route_returns_json_404() {
    let reply = get(common::test_state().await, "/api").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
    let json: serde_json::Value = serde_json::from_slice(&reply.body).expect("parsear JSON");
    assert_eq!(json["error"], "not_found");
}

#[tokio::test]
async fn malformed_api_paths_are_classified_as_api() {
    // Claves que, normalizadas, quedan bajo `api/`: la API nunca debe caer al
    // fallback SPA. Se usa URI absoluta porque `//api/health` es ambiguo a
    // nivel de `Uri` (se interpretaría como autoridad).
    for uri in [
        "http://localhost//api/health",
        "http://localhost/./api/health",
    ] {
        let reply = get(common::test_state().await, uri).await;

        assert_eq!(reply.status, StatusCode::NOT_FOUND, "uri {uri}");
        assert!(
            header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
            "{uri}: content-type inesperado: {:?}",
            reply.headers.get(header::CONTENT_TYPE)
        );
        let text = body_lower(&reply);
        assert!(
            !text.contains("<html") && !text.contains("<!doctype"),
            "{uri}: no debe devolver index.html: {text}"
        );
    }
}

#[tokio::test]
async fn traversal_with_percent_encoding_does_not_leak() {
    let reply = get(common::test_state().await, "/%2e%2e/%2e%2e/etc/passwd").await;

    let text = body_lower(&reply);
    assert!(
        !text.contains("root:"),
        "no debe filtrar /etc/passwd: {text}"
    );
    // Sin fuga: o bien index.html (embed-ui) o el 404 JSON (modo stub).
    assert!(
        reply.status == StatusCode::NOT_FOUND
            || header_str(&reply.headers, header::CONTENT_TYPE).starts_with("text/html"),
        "respuesta inesperada: {} {:?}",
        reply.status,
        reply.headers.get(header::CONTENT_TYPE)
    );
}

#[tokio::test]
async fn unknown_extension_top_level_returns_404() {
    let reply = get(common::test_state().await, "/foo.unknownext").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let text = body_lower(&reply);
    assert!(
        !text.contains("<html") && !text.contains("<!doctype"),
        "un asset huérfano no debe devolver index.html: {text}"
    );
}

// ---------------------------------------------------------------------------
// Con la feature `embed-ui`
// ---------------------------------------------------------------------------

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn root_serves_index_html() {
    let reply = get(common::test_state().await, "/").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("text/html"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );

    let text = body_lower(&reply);
    assert!(
        text.contains("<html") || text.contains("<!doctype"),
        "el cuerpo no parece HTML: {text}"
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn spa_route_serves_index_html() {
    let index = get(common::test_state().await, "/").await;
    let reply = get(common::test_state().await, "/shopping").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("text/html"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
    assert_eq!(
        reply.body, index.body,
        "una ruta SPA debe devolver el mismo index.html que /"
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn hashed_asset_is_served_with_immutable_cache() {
    // Buscamos un asset real en `frontend/dist/assets`; si no hay (placeholder
    // de build.rs), validamos la lógica de cache con `index.html`.
    let assets_dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/dist/assets"
    ));

    let asset = std::fs::read_dir(assets_dir).ok().and_then(|entries| {
        entries
            .filter_map(Result::ok)
            .find(|entry| entry.path().is_file())
    });

    match asset {
        Some(entry) => {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let reply = get(common::test_state().await, &format!("/assets/{name}")).await;

            assert_eq!(reply.status, StatusCode::OK, "asset {name} no servido");
            assert!(
                header_str(&reply.headers, header::CACHE_CONTROL).contains("immutable"),
                "cache-control inesperado: {:?}",
                reply.headers.get(header::CACHE_CONTROL)
            );
            assert!(
                !header_str(&reply.headers, header::CONTENT_TYPE).is_empty(),
                "el asset debe llevar content-type"
            );
        }
        None => {
            let reply = get(common::test_state().await, "/").await;
            assert_eq!(reply.status, StatusCode::OK);
            assert_eq!(
                header_str(&reply.headers, header::CACHE_CONTROL),
                "no-cache",
                "index.html debe revalidarse"
            );
        }
    }
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn head_root_returns_200_without_body() {
    let reply = request(common::test_state().await, "HEAD", "/").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("text/html"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
    assert!(reply.body.is_empty(), "HEAD no debe llevar cuerpo");
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn post_to_spa_route_is_method_not_allowed() {
    let reply = request(common::test_state().await, "POST", "/shopping").await;

    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
    let json: serde_json::Value = serde_json::from_slice(&reply.body).expect("parsear JSON");
    assert_eq!(json["error"], "method_not_allowed");
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn assets_trailing_slash_serves_index_not_404() {
    // `/assets/` normaliza a `assets` (sin extensión): es una ruta de cliente,
    // no un asset huérfano.
    let reply = get(common::test_state().await, "/assets/").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("text/html"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn dotted_client_route_serves_index_html() {
    let index = get(common::test_state().await, "/").await;
    let reply = get(common::test_state().await, "/recipe/pasta.v2").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.body, index.body,
        "una ruta de cliente con punto debe devolver index.html"
    );
}

// ---------------------------------------------------------------------------
// Sin la feature `embed-ui`
// ---------------------------------------------------------------------------

#[cfg(not(feature = "embed-ui"))]
#[tokio::test]
async fn root_without_embed_ui_returns_404_with_hint() {
    let reply = get(common::test_state().await, "/").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );

    let json: serde_json::Value = serde_json::from_slice(&reply.body).expect("parsear JSON");
    assert_eq!(json["error"], "not_found");
    let message = json["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("UI no empaquetada"),
        "mensaje inesperado: {message}"
    );
}

#[cfg(not(feature = "embed-ui"))]
#[tokio::test]
async fn spa_route_without_embed_ui_returns_404_json() {
    let reply = get(common::test_state().await, "/shopping").await;

    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(
        header_str(&reply.headers, header::CONTENT_TYPE).starts_with("application/json"),
        "content-type inesperado: {:?}",
        reply.headers.get(header::CONTENT_TYPE)
    );
}

// ---------------------------------------------------------------------------
// Hardening del servido estático (ETag / charset / cabeceras de seguridad)
// ---------------------------------------------------------------------------

/// Ejecuta una petición con cabeceras extra contra el router completo.
#[cfg(feature = "embed-ui")]
async fn request_with_headers(
    state: AppState,
    method: &str,
    uri: &str,
    extra: &[(&str, &str)],
) -> Reply {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in extra {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(Body::empty()).expect("construir request");
    let response = app(state).oneshot(request).await.expect("ejecutar request");

    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes()
        .to_vec();

    Reply {
        status,
        headers,
        body,
    }
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn index_includes_etag_charset_and_security_headers() {
    let reply = get(common::test_state().await, "/").await;

    assert_eq!(reply.status, StatusCode::OK);
    let etag = header_str(&reply.headers, header::ETAG);
    assert!(!etag.is_empty(), "index.html debe incluir ETag");
    assert!(
        etag.starts_with('"') && etag.ends_with('"'),
        "ETag entre comillas: {etag}"
    );
    assert_eq!(
        header_str(&reply.headers, header::CONTENT_TYPE),
        "text/html; charset=utf-8"
    );
    assert_eq!(
        header_str(&reply.headers, header::X_CONTENT_TYPE_OPTIONS),
        "nosniff"
    );
    assert!(
        header_str(&reply.headers, header::CONTENT_SECURITY_POLICY).contains("script-src 'self'"),
        "CSP inesperada: {:?}",
        reply.headers.get(header::CONTENT_SECURITY_POLICY)
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn index_revalidation_with_matching_etag_returns_304() {
    let first = get(common::test_state().await, "/").await;
    let etag = first
        .headers
        .get(header::ETAG)
        .expect("ETag")
        .to_str()
        .expect("ascii")
        .to_string();

    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        "/",
        &[("if-none-match", &etag)],
    )
    .await;

    assert_eq!(reply.status, StatusCode::NOT_MODIFIED);
    assert!(reply.body.is_empty(), "304 no debe llevar cuerpo");
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn index_revalidation_with_stale_etag_returns_200() {
    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        "/",
        &[("if-none-match", "\"no-coincide\"")],
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(!reply.body.is_empty(), "200 debe llevar cuerpo");
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn index_revalidation_with_star_returns_304() {
    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        "/",
        &[("if-none-match", "*")],
    )
    .await;

    assert_eq!(reply.status, StatusCode::NOT_MODIFIED);
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn range_request_is_served_full_without_accept_ranges() {
    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        "/",
        &[("range", "bytes=0-3")],
    )
    .await;

    assert_eq!(
        reply.status,
        StatusCode::OK,
        "Range no soportado: 200 completo"
    );
    assert!(
        reply.headers.get(header::ACCEPT_RANGES).is_none(),
        "no debe anunciarse Accept-Ranges"
    );
    assert!(!reply.body.is_empty());
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn hashed_asset_has_etag_nosniff_no_csp_and_revalidates() {
    let assets_dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/dist/assets"
    ));
    let asset = std::fs::read_dir(assets_dir).ok().and_then(|entries| {
        entries
            .filter_map(Result::ok)
            .find(|entry| entry.path().is_file())
    });

    let Some(entry) = asset else {
        // Sin assets reales (placeholder), se valida sobre /assets/ index.html.
        let reply = get(common::test_state().await, "/").await;
        assert!(!header_str(&reply.headers, header::ETAG).is_empty());
        return;
    };

    let name = entry.file_name();
    let name = name.to_string_lossy();
    let uri = format!("/assets/{name}");
    let first = get(common::test_state().await, &uri).await;

    assert_eq!(first.status, StatusCode::OK, "asset {name} no servido");
    assert!(
        !header_str(&first.headers, header::ETAG).is_empty(),
        "asset debe traer ETag"
    );
    assert_eq!(
        header_str(&first.headers, header::X_CONTENT_TYPE_OPTIONS),
        "nosniff"
    );
    assert!(
        first.headers.get(header::CONTENT_SECURITY_POLICY).is_none(),
        "un asset no-HTML no debe llevar CSP"
    );

    let etag = first
        .headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        &uri,
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(
        reply.status,
        StatusCode::NOT_MODIFIED,
        "asset debe revalidar a 304"
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn index_revalidation_with_etag_list_returns_304() {
    let first = get(common::test_state().await, "/").await;
    let etag = first
        .headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let list = format!("\"otro\", {etag}");

    let reply = request_with_headers(
        common::test_state().await,
        "GET",
        "/",
        &[("if-none-match", &list)],
    )
    .await;
    assert_eq!(
        reply.status,
        StatusCode::NOT_MODIFIED,
        "lista de ETags debe revalidar"
    );
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn head_index_with_matching_etag_returns_304_without_body() {
    let first = get(common::test_state().await, "/").await;
    let etag = first
        .headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let reply = request_with_headers(
        common::test_state().await,
        "HEAD",
        "/",
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(reply.status, StatusCode::NOT_MODIFIED);
    assert!(reply.body.is_empty(), "HEAD 304 no debe llevar cuerpo");
}

#[cfg(feature = "embed-ui")]
#[tokio::test]
async fn missing_asset_error_has_nosniff() {
    let reply = get(common::test_state().await, "/assets/no-existe-abc.js").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(
        header_str(&reply.headers, header::X_CONTENT_TYPE_OPTIONS),
        "nosniff"
    );
}
