# Change: Hardening del servido estático

## Why
La auditoría de Fase 2 (`rust-reviewer`) detectó dos carencias en el servido del frontend (`backend/src/static_files/`):

- `index.html` se sirve con `cache-control: no-cache` (correcto) pero sin `ETag`/`Last-Modified`: cada revalidación cuesta siempre un `200` con el cuerpo completo, y no se atiende `If-None-Match` (issue #8).
- Los tipos MIME textuales (`text/html`, `text/css`, `text/javascript`) se sirven sin `; charset=utf-8`; no se envían `X-Content-Type-Options: nosniff` ni `Content-Security-Policy` (issue #9).

## What Changes
- **ETag + revalidación (#8):** incluir una cabecera `ETag` fuerte, derivada del hash SHA-256 que `rust-embed` ya calcula en tiempo de compilación (`EmbeddedFile::metadata.sha256_hash()`), en toda respuesta estática, y responder `304 Not Modified` (sin cuerpo) cuando `If-None-Match` coincida.
- **Charset (#9):** añadir `; charset=utf-8` a los tipos `text/*`.
- **Cabeceras de seguridad (#9):** añadir `X-Content-Type-Options: nosniff` a todas las respuestas estáticas y `Content-Security-Policy` a los documentos HTML, con una política compatible con Ant Design (CSS-in-JS → `style-src 'unsafe-inline'`).
- **Decisión sobre `Range` (#9):** NO implementar `Range`/`Accept-Ranges` (activos de un SPA, inmutables y pequeños); se documenta la decisión en `design.md` y se cubre con un test.

## Impact
- Capability afectada: `frontend-serving` (requisitos AÑADIDOS; no se modifica ninguno existente).
- Código: `backend/src/static_files/embedded.rs` (único punto de respuesta estática).
- Tests: `backend/tests/static_files.rs` + tests unitarios en `embedded.rs`.
- Sin cambios de contrato de la API, de la base de datos ni del frontend.
