# Design — Hardening del servido estático

## Context
`backend/src/static_files/embedded.rs` sirve el frontend embebido (`rust-embed`) tras el fallback del router. Los activos bajo `assets/` llevan hash en el nombre y se cachean `immutable`; el resto (`index.html`, favicon…) se sirve `no-cache` pero sin validador, así que no hay revalidación barata. No se envían cabeceras de seguridad ni charset.

## Goals / Non-Goals
- **Goals:** revalidación con `ETag`/`If-None-Match` (304), `charset=utf-8` en tipos textuales, `nosniff` + CSP, y dejar constancia explícita de la decisión sobre `Range`.
- **Non-Goals:** soporte de `Range`/`Accept-Ranges`; `Last-Modified`; CSP por nonce/hash; cambios en el modo sin `embed-ui` (stub) más allá de lo aquí descrito; cabeceras de seguridad en las respuestas de la API.

## Decisions

### D1 — Origen del `ETag`: hash SHA-256 embebido
`rust-embed` calcula el SHA-256 de cada fichero en compilación y lo expone en `EmbeddedFile::metadata.sha256_hash() -> [u8;32]`. El `ETag` será un validador **fuerte** `"<64 hex minúsculas>"`, estable entre reinicios y builds idénticos y sin coste de hashing en runtime (los activos grandes como `antd-*.js` ~694 kB no se re-hashean por petición). Se descarta hashear el cuerpo en runtime y `Last-Modified` (dependiente del FS, no determinista).

### D2 — Charset: solo tipos `text/*`
La regla es: si la esencia MIME del `Content-Type` empieza por `text/`, se le añade `; charset=utf-8`. No se toca `image/*`, `application/octet-stream`, etc. `mime_guess` ya devuelve `text/html`, `text/css`, `text/javascript`.

### D3 — CSP: política compatible con el frontend real
El HTML compilado no contiene scripts ni estilos inline (verificado en `frontend/dist/index.html`: solo `<script type="module">` y `<link rel="stylesheet">` con `src/href`). Ant Design usa CSS-in-JS e inyecta `style=""` en nodos del DOM, lo que exige `style-src 'unsafe-inline'`. Política:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'; manifest-src 'self'; worker-src 'self' blob:
```

Se aplica **solo** a respuestas con `Content-Type: text/html` (es donde el navegador la interpreta); `X-Content-Type-Options: nosniff` se aplica a **todas** las respuestas estáticas.

### D4 — `Range`: no soportado (decisión)
Los activos son inmutables (`assets/` con hash) y pequeños; `index.html` es de pocos kB. El coste/complejidad de `Range` (multipart, `HEAD`, combinaciones) no aporta valor a un SPA. Por tanto NO se anuncia `Accept-Ranges` y una petición con `Range:` se sirve como `200` con el cuerpo completo. Se cubre con test para fijar la decisión.

### D5 — Alcance: solo la implementación `embed-ui`
Los cambios viven en `embedded.rs`. El modo sin `embed-ui` (`stub.rs`) no sirve activos y queda igual.

## Contracts (Rust)
- `fn asset_response(key: &str, data: Cow<'static, [u8]>, sha256: [u8; 32]) -> Response`
- `fn etag_value(sha256: &[u8; 32]) -> HeaderValue` — `"<hex>"` (32 bytes → 64 hex).
- `fn content_type_for(key: &str) -> HeaderValue` — MIME + `; charset=utf-8` si `text/*`.
- `fn cache_control_for(key: &str) -> &'static str` — `immutable` si `assets/`, si no `no-cache`.
- `fn apply_security_headers(headers: &mut HeaderMap, is_html: bool)` — `nosniff` siempre; CSP si `is_html`.
- `fn if_none_match_matches(headers: &HeaderMap, etag: &HeaderValue) -> bool` — compara tokens separados por coma/espacios; `*` coincide.
- `pub async fn serve(method: Method, headers: HeaderMap, uri: Uri) -> Response` — añade `HeaderMap`.

## Testing
- **Unit** (`embedded.rs`): `etag_value` determinista; `content_type_for` con charset para `text/*`; `cache_control_for`; `apply_security_headers` (nosniff siempre, CSP solo HTML); `if_none_match_matches` (igual, lista, `*`, distinto, ausente).
- **Integración** (`tests/static_files.rs`, feature `embed-ui`): `GET /` trae `ETag` y `charset`; `GET /` con `If-None-Match` igual → `304` sin cuerpo; distinto → `200`; `*` → `304`; `GET /assets/<real>` revalida → `304`; CSP presente en `/` y ausente en un asset `.js`; `nosniff` presente; `Range` → `200` completo sin `Accept-Ranges`.
