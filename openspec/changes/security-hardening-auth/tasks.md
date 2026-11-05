# Tasks — Endurecimiento del BFF de sesión y cliente OIDC (#81)

## 1. RED (tests que fallan)
- [ ] 1.1 `Debug` redacta secretos: `format!("{:?}", config)` y de `OidcConfig`/`SessionConfig` no contienen `client_secret` ni `SESSION_SECRET` (`backend/src/config.rs`)
- [ ] 1.2 Nombre de cookie por defecto `__Host-oximeal_session` y validación: un `__Host-*` sin `Secure` ⇒ `Err` al cargar config
- [ ] 1.3 `SESSION_COOKIE_SECURE=false` + `OIDC_ENABLED=true` ⇒ `Config::from_env()` es `Err`; con `ALLOW_INSECURE_COOKIE=true` ⇒ `Ok` con warning
- [ ] 1.4 `verify_id_token` rechaza `nbf` en el futuro y `azp` distinto del `client_id` (mock de `common/idp.rs`)
- [ ] 1.5 Caducidad absoluta: sesión con `started_at` anterior a `SESSION_ABSOLUTE_TTL_SECS` ⇒ 401 en ruta protegida y `authenticated: false` en `/auth/me`; y no se refresca
- [ ] 1.6 Respuestas autenticadas (cookie y Bearer) llevan `Cache-Control: no-store`; las públicas no lo exigen
- [ ] 1.7 `cargo test` → rojo solo en lo nuevo; los tests vigentes siguen verdes

## 2. GREEN (implementación mínima)
- [ ] 2.1 `impl Debug` manual para `Config`, `OidcConfig` y `SessionConfig` con `[REDACTED]`
- [ ] 2.2 `SessionConfig.absolute_ttl` (`SESSION_ABSOLUTE_TTL_SECS`, 12 h) y `AuthenticatedSession.started_at`
- [ ] 2.3 `config.rs`: default `__Host-oximeal_session`, validación del prefijo y fail-closed con `ALLOW_INSECURE_COOKIE`
- [ ] 2.4 `oidc.rs`: `validation.validate_nbf = true` (con leeway) y comprobación de `azp`
- [ ] 2.5 `middleware.rs`/`routes.rs`: comprobar la vida absoluta y destruir la sesión (401 / `authenticated: false`)
- [ ] 2.6 Capa única de `Cache-Control: no-store` para respuestas autenticadas (cookie y Bearer)
- [ ] 2.7 `cargo test` → verde

## 3. REFACTOR
- [ ] 3.1 `cargo fmt` / `cargo check`
- [ ] 3.2 `cargo clippy --all-targets --all-features -- -D warnings` → 0 avisos
- [ ] 3.3 `cargo test` con y sin la feature `embed-ui` → verde

## 4. Documentación y cierre
- [ ] 4.1 `deploy/README.md` y `.env.example`: `__Host-`, `SESSION_ABSOLUTE_TTL_SECS`, `ALLOW_INSECURE_COOKIE`, HSTS/`Referrer-Policy` en el proxy
- [ ] 4.2 `CHANGELOG.md` (`[No publicado]`) con el impacto de despliegue (sesiones existentes invalidadas por el cambio de nombre de cookie)
- [ ] 4.3 `just ci` y `just ci-ui` en verde
- [ ] 4.4 `openspec validate --all --strict` → verde
- [ ] 4.5 Archivar el change y PR `feature/security-hardening-auth` → `development`
