# Tasks — Shell PWA instalable (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Leer `frontend/index.html`, `backend/src/static_files/embedded.rs` y la CSP
- [x] 1.2 Generar el icono maestro y derivar 192/512/maskable/apple-touch con `magick`

## 2. RED
- [x] 2.1 Test (backend): `GET /manifest.webmanifest` responde 200 con `Content-Type: application/manifest+json`
- [x] 2.2 Test (backend): `sw.js` se sirve con `Cache-Control: no-cache`
- [x] 2.3 Test (frontend): el registro no se invoca fuera de producción
- [x] 2.4 Test (frontend): el manifiesto es JSON válido con los campos obligatorios
- [x] 2.5 Ejecutar `just ci` y `pnpm test`; confirmar el estado RED

## 3. GREEN
- [x] 3.1 `frontend/public/manifest.webmanifest` y los iconos
- [x] 3.2 `frontend/public/sw.js`
- [x] 3.3 `frontend/src/offline/register-sw.ts` y su llamada en `main.tsx`
- [x] 3.4 `frontend/index.html` (manifest, favicon, apple-touch-icon)
- [x] 3.5 MIME del `.webmanifest` en `backend/src/static_files/embedded.rs`
- [x] 3.6 `just ci` y `pnpm test` en verde

## 4. REFACTOR
- [x] 4.1 `cargo fmt --check`, `cargo clippy -- -D warnings`, `pnpm lint` y `pnpm format:check` sin hallazgos
- [x] 4.2 Verificar `pnpm check-bundle` (presupuesto intacto)

## 5. Cierre
- [x] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [x] 5.2 Actualizar `PLAN.md` §18.3 y `CHANGELOG.md`; archivar el change
