# Tasks — Fase 4: Autenticación OIDC con Pocket ID (TDD: RED → GREEN → REFACTOR)

## 1. Dependencias y configuración
- [ ] 1.1 Añadir dependencias backend (`openidconnect`/`oauth2`, `jsonwebtoken`, `tower-sessions` o equivalente, `sha2`, `rand`) y verificar que el workspace compila.
- [ ] 1.2 Extender `config.rs` con `OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `ALLOWED_ORIGINS`, `OIDC_ALLOWED_GROUPS` y `SESSION_*`.
- [ ] 1.3 Migración nueva con la tabla de sesiones server-side (sin tocar las tablas de dominio).
- [ ] 1.4 Fallo de arranque con config incompleta o secreto de sesión por defecto; warning explícito con `OIDC_ENABLED=false`.

## 2. BFF y sesión
- [ ] 2.1 `GET /auth/login`: redirección con `response_type=code`, `scope`, `resource`, `state`, `nonce` y PKCE `S256` (test primero).
- [ ] 2.2 `GET /auth/callback`: validación de `state`, intercambio de código en servidor, validación del ID token y creación de sesión con rotación de ID.
- [ ] 2.3 Cookie `httpOnly` + `Secure` + `SameSite=Lax` + `Path=/`; sin exponer tokens al navegador.
- [ ] 2.4 `GET /auth/me` con `{ authenticated, subject, name, permissions, csrf_token }` y token anti-CSRF; nunca expone tokens.
- [ ] 2.5 `POST /auth/logout`: destrucción de sesión, limpieza de cookie y 204.
- [ ] 2.6 Renovación transparente con refresh token y destrucción de sesión ante fallo (401).
- [ ] 2.7 Middleware CSRF `X-CSRF-Token` en mutaciones autenticadas por cookie.

## 3. Validación de tokens y autorización
- [ ] 3.1 Middleware de validación de access tokens: JWKS con caché y refresco ante `kid` desconocido, `iss`, `aud`, `exp`/`nbf` y algoritmo permitido (rechazo de `none`).
- [ ] 3.2 Autorización por permiso por endpoint (`read:recipes`, `write:recipes`, `read:plan`, `write:plan`, `read:shopping`, `write:shopping`): 401 con `WWW-Authenticate: Bearer` sin credenciales, 403 sin permiso.
- [ ] 3.3 Rutas públicas sin cambios: `GET /api/health`, OpenAPI y Swagger UI.
- [ ] 3.4 Sustituir CORS `Any` por `ALLOWED_ORIGINS`; sin `allow_origin(Any)` con credenciales.
- [ ] 3.5 Autorización indistinta por cookie de sesión o `Authorization: Bearer`.

## 4. Agent API M2M
- [ ] 4.1 Proteger los cinco endpoints `/api/agent/*` con los permisos `agent:plan`, `agent:shopping` y `agent:read`.
- [ ] 4.2 Verificar que los tokens M2M (Valet) acceden y que los tokens user-delegated no heredan permisos `agent:*`.
- [ ] 4.3 Confirmar que el contrato de peticiones/respuestas de la Agent API no cambia.
- [ ] 4.4 Retirar el API key estático (`OXIMEAL_API_KEY`, issue #5) y documentar la migración.

## 5. Frontend
- [ ] 5.1 Contexto de sesión que consulta `GET /auth/me` al arrancar (estado de carga sin renderizar datos).
- [ ] 5.2 Guard de rutas: sin sesión, redirección a `/auth/login` conservando el destino.
- [ ] 5.3 Manejo de 401 (re-login) y 403 (aviso sin destruir sesión).
- [ ] 5.4 Cabecera `X-CSRF-Token` en todas las mutaciones.
- [ ] 5.5 Identidad de sesión visible: nombre del usuario y control de cierre de sesión; sin roles ni acciones de administración.

## 6. Despliegue local (IdP de desarrollo)
- [ ] 6.1 `compose` (podman/docker) con Pocket ID + su base de datos para desarrollo.
- [ ] 6.2 Documentar la creación del API (resource `https://oximeal.local/api`), las permission keys y los permisos.
- [ ] 6.3 Documentar el cliente user-delegated (SPA, *Allowed user groups*) y el cliente M2M (Valet, *Client access*).
- [ ] 6.4 Documentar HTTPS por reverse proxy para redirect URIs fuera de localhost.

## 7. Tests y CI
- [ ] 7.1 Tests de integración del flujo BFF (login feliz, `state` inválido, usuario fuera de grupos, logout, refresh fallido, CSRF).
- [ ] 7.2 Tests de validación de tokens (firma/clave desconocida, caducado, `aud` ajeno, algoritmo no permitido).
- [ ] 7.3 Tests de autorización por permiso (401/403/200) y de endpoints públicos.
- [ ] 7.4 Tests de la Agent API M2M y no-regresión del contrato.
- [ ] 7.5 Tests de frontend (guard de rutas, 401/403, CSRF, identidad y cierre de sesión).
- [ ] 7.6 `cargo test` en verde con `OIDC_ENABLED=false`; `clippy -D warnings` y `fmt --check` limpios.
- [ ] 7.7 `openspec validate oidc-auth --strict` y `just ci` / `just ci-ui` en verde.

## 8. Documentación
- [ ] 8.1 Actualizar `.env.example` y README con las variables OIDC, `ALLOWED_ORIGINS`, `OIDC_ALLOWED_GROUPS` y `SESSION_*`.
- [ ] 8.2 Documentar los permisos por endpoint y la allowlist de grupos permitidos.
- [ ] 8.3 Documentar la migración del API key estático a M2M (issue #5) y los pasos del `compose` de desarrollo.
- [ ] 8.4 Anotar la dependencia de la Fase 5 (offline con sesión potencialmente caducada).
