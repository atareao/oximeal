# Tasks — Hardening del servido estático (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Confirmar línea base: `cargo test -p oximeal-backend --features embed-ui` en verde y `openspec validate --all --strict`
- [x] 1.2 Localizar los puntos de respuesta estática en `backend/src/static_files/embedded.rs` (`asset_response`, fallback SPA, 404 huérfano)

## 2. ETag e If-None-Match (issue #8)
- [x] 2.1 RED: test unitario de que `etag_value` produce `"<64 hex>"` determinista
- [x] 2.2 RED: test de integración `GET /` trae `ETag`; `If-None-Match` igual → `304` sin cuerpo; distinto → `200`; `*` → `304`
- [x] 2.3 RED: test de integración de revalidación de un asset real bajo `/assets/`
- [x] 2.4 GREEN: `ETag` desde `metadata.sha256_hash()` y `serve` atiende `If-None-Match`
- [x] 2.5 REFACTOR: helper `if_none_match_matches` (lista separada por comas, `*`)

## 3. Charset y cabeceras de seguridad (issue #9)
- [x] 3.1 RED: test unitario de `; charset=utf-8` en tipos `text/*`
- [x] 3.2 RED: test unitario de `X-Content-Type-Options: nosniff` en todas las respuestas
- [x] 3.3 RED: test de `Content-Security-Policy` presente en HTML y ausente en no-HTML
- [x] 3.4 GREEN: `content_type_for` + `apply_security_headers` aplicados en `asset_response`

## 4. Decisión Range y cierre
- [x] 4.1 RED: test de que `Range` devuelve `200` completo y sin `Accept-Ranges`
- [x] 4.2 GREEN/REFACTOR: `serve` recibe `HeaderMap`; decisión documentada en `design.md`
- [x] 4.3 `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test -p oximeal-backend` (con y sin `embed-ui`)
- [x] 4.4 `openspec validate static-serving-hardening --strict` y `openspec validate --all --strict`
- [ ] 4.5 Actualizar `PLAN.md` y cerrar las issues #8 y #9 (tras merge)
