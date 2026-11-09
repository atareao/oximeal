# Tasks

## 1. Backend — RP-initiated logout (TDD)

### RED

- [x] 1.1 Tests de `POST /auth/logout`: responde 200 con `{"logout_url": ...}` y destruye la sesión/limpia la cookie.
- [x] 1.2 Test de construcción de URL: incluye `id_token_hint` (cuando la sesión tiene `id_token`), `client_id` y `post_logout_redirect_uri` con codificación de query correcta.
- [x] 1.3 Tests de fallback `logout_url: null`: IdP sin `end_session_endpoint` y petición sin sesión autenticada.
- [x] 1.4 Test de CSRF intacto: `POST /auth/logout` sin `X-CSRF-Token` con sesión cookie responde 403.
- [x] 1.5 Ejecutar `cargo test` y confirmar RED.

### GREEN

- [x] 1.6 Actualizar el handler `logout` en `backend/src/auth/routes.rs`: destruir sesión, limpiar cookie y responder 200 con `logout_url`.
- [x] 1.7 Añadir en `backend/src/auth/oidc.rs` el helper de construcción de la URL de fin de sesión (params codificados; `None` si no aplica).
- [x] 1.8 Ejecutar `cargo test` y confirmar GREEN.

### REFACTOR

- [x] 1.9 `cargo clippy -- -D warnings` sin warnings.
- [x] 1.10 `cargo fmt --check` limpio.
- [x] 1.11 Revisar que no se registran tokens en logs ni se añaden al cuerpo.
- [x] 1.12 Endurecer `logout`: destruir sesión/cookie antes del I/O de red, `timeout` en `discovery()` y `connect_timeout`/`timeout` en el `reqwest::Client` de OIDC.
- [x] 1.13 Validar esquema `http`/`https` en `logout_url` y `build_authorize_url`; tests de discovery caído, modo abierto (404) y sesión sin `id_token`.

## 2. Frontend — navegación a `logout_url` (TDD)

### RED

- [x] 2.1 Definir/ajustar el tipo de la respuesta de `logoutAuth` (`{ logout_url: string | null }`).
- [x] 2.2 Tests de `SessionProvider`: con `logout_url` limpia estado/caché y navega con `window.location.assign`.
- [x] 2.3 Tests de `SessionProvider`: con `logout_url: null` limpia estado/caché y queda anónimo (login), sin navegación externa.
- [x] 2.4 Tests de `AppHeader`: `Salir` autenticado delega en el logout; sin sesión reinicia sin `POST`.
- [x] 2.5 Ejecutar `npx vitest run` y confirmar RED.

### GREEN

- [x] 2.6 Implementar en `SessionProvider.logout` la lectura de `logout_url`, la limpieza de estado/caché y `window.location.assign`.
- [x] 2.7 Mantener el fallback actual (anónimo → login) cuando `logout_url` es `null`.
- [x] 2.8 `npx tsc --noEmit` sin errores y `npx vitest run` en verde.

### REFACTOR

- [x] 2.9 `npx eslint .` sin errores.
- [x] 2.10 `npx prettier --check` limpio.
- [x] 2.11 Re-ejecutar `npx vitest run` para descartar regresiones.

## 3. Verificación y entrega

- [x] 3.1 `npx @fission-ai/openspec@1.16.0 validate --all --strict` sin errores.
- [x] 3.2 Auditoría `@rust-reviewer` sobre los cambios de backend.
- [x] 3.3 Auditoría `@react-reviewer` sobre los cambios de frontend.
- [x] 3.4 Preparar release 0.8.1.
- [x] 3.5 Desplegar en co1 con Dockge Update y verificar el cierre de sesión completo (IdP incluido). Verificado el 2026-11-07: tras `Salir` se vuelve al login pasando por Pocket ID, sin reentrada automática.
- [x] 3.6 Actualizar `docs/` y archivar el change.