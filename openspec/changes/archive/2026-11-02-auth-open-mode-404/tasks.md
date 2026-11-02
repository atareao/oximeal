# Tasks — Modo abierto: `/auth/*` fuera del fallback SPA (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Leer `backend/src/static_files.rs` (`is_api_path`, `normalize`) y el fallback `ui::serve`
- [x] 1.2 Localizar los tests existentes de clasificación de rutas API y del fallback SPA en `backend/tests/`

## 2. RED
- [x] 2.1 Test: con OIDC desactivado, `GET /auth/me` responde 404 JSON y no `index.html`
- [x] 2.2 Test: con OIDC desactivado, `GET /auth/login` responde 404 JSON
- [x] 2.3 Test: con OIDC activo, una ruta `/auth/*` inexistente responde 404 JSON
- [x] 2.4 Test: las rutas del cliente (`/shopping`, `/recetas/pasta.v2`) siguen sirviendo `index.html`
- [x] 2.5 Ejecutar `cargo test -p oximeal-backend` y confirmar el estado RED

## 3. GREEN
- [x] 3.1 `is_api_path` clasifica `auth` y `auth/*` como rutas de API
- [x] 3.2 Ejecutar `cargo test -p oximeal-backend` y `cargo test -p oximeal-backend --features embed-ui`
- [x] 3.3 Verificar a mano con el binario real que la app funciona en modo abierto
- [x] 3.4 Acortar el requisito «Callback y sesión server-side» de `auth-session` para que `openspec validate --all --strict` pase (alineado con `development`)

## 4. REFACTOR
- [x] 4.1 `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`

## 5. Cierre
- [x] 5.1 `just ci` y `openspec validate --all --strict` en verde
- [x] 5.1b Llevar a `main` el requisito acortado de `auth-session` (ya está en `development`)
- [x] 5.2 Actualizar `CHANGELOG.md`; archivar el change
