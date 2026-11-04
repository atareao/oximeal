# Tasks — Destino de retorno seguro tras el login (#80)

## 1. RED (tests que fallan)
- [x] 1.1 Ampliar `sanitize_return_to_rejects_external` (`backend/src/auth/mod.rs:369`) con: `%09` (TAB), `%0a` (LF), `%0d` (CR), `"/\t/evil.com"`, `"/\n/evil.com"` → deben devolver `None`
- [x] 1.2 Test de que se siguen aceptando rutas legítimas: `/shopping`, `/recipes?q=pasta` (con `?`, `&`, espacio) → `Some`
- [x] 1.3 Test de regresión del sink: un valor aceptado nunca produce un `Location` fuera del origen (no empieza por `//` ni es absoluto)
- [x] 1.4 Test de integración del login (`GET /auth/login?return_to=/%09/evil.com`): el `return_to` guardado en la sesión es `/` (si el arnés lo permite)
- [x] 1.5 `cargo test -p oximeal-backend auth` → **rojo** solo en los casos nuevos (los vigentes siguen verdes)

## 2. GREEN (implementación mínima)
- [x] 2.1 `sanitize_return_to`: vetar `value.chars().any(char::is_control)` antes del resto de comprobaciones
- [x] 2.2 `cargo test -p oximeal-backend auth` → **verde** (nuevos + vigentes)

## 3. REFACTOR
- [x] 3.1 `cargo fmt` / `cargo check`
- [x] 3.2 `cargo clippy --all-targets --all-features -- -D warnings` → 0 avisos
- [x] 3.3 `cargo test` con y sin la feature `embed-ui` → verde

## 4. Puertas y cierre
- [x] 4.1 `just ci` y `just ci-ui` en verde
- [x] 4.2 `openspec validate --all --strict` → 28/28 (27 specs + change)
- [x] 4.3 Documentar en el `CHANGELOG` (`[No publicado]` > Corregido, ref. #80)
- [x] 4.4 Archivar el change (`openspec archive fix-return-to-open-redirect`) y PR a `main` (PR #82)
- [x] 4.5 Sincronizar `main` → `development` (PR #83)
- [x] 4.6 Cerrar #80 (cerrado por `Closes #80`)
