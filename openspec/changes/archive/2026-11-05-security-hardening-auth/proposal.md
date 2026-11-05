# Change: endurecimiento del BFF de sesión y del cliente OIDC

## Why

Backlog de endurecimiento de la auditoría de seguridad run-1 (issue **#81**). Son
**notas de defensa en profundidad**: ninguna es explotable por sí sola (el único
hallazgo explotable fue #80, ya corregido en v0.5.1). Se agrupan aquí las seis que
tocan el BFF de sesión y el cliente OIDC porque comparten superficie y tests.

Estado actual verificado en código:

- La cookie de sesión se llama `oximeal_session` (`config.rs:22`) y se emite con
  `Secure`/`SameSite=Lax`/`Path=/` pero **sin** prefijo `__Host-` (`lib.rs:56-66`),
  así que un subdominio comprometido puede hacer *cookie tossing* (DoS: la cookie
  inválida eclipsa la legítima; la firma impide forjar sesión).
- `SESSION_COOKIE_SECURE=false` con OIDC activo solo emite un **warning**
  (`config.rs:139-145`): bajarlo en producción sobre HTTP habilita *hijack*.
- `Expiry::OnInactivity(ttl)` (`lib.rs:65`) ⇒ una sesión con `offline_access`
  puede vivir indefinidamente mientras haya actividad; no hay **caducidad
  absoluta**.
- `verify_id_token` valida firma, `iss`, `aud`, `exp` y `nonce` (`oidc.rs:207-248`)
  pero **no** `nbf` ni `azp`.
- Las respuestas autenticadas no llevan `Cache-Control: no-store`
  (`static_files` solo gestiona el servido de la SPA): un proxy o el propio
  navegador pueden almacenarlas.
- `Config`, `OidcConfig` y `SessionConfig` derivan `Debug` (`config.rs:41,63,87`)
  e incluyen `client_secret` y `SESSION_SECRET`: un `{:?}` o un `tracing` con
  `?config` los volcaría en los logs.

## What Changes

1. **Cookie `__Host-`**: el nombre por defecto pasa a `__Host-oximeal_session`.
   El prefijo solo es válido con `Secure` + `Path=/` y **sin** `Domain`; se
   valida el nombre configurado contra esas reglas y se falla el arranque si el
   operador pone un `__Host-*` sin `Secure`.
2. **Fail-closed del flag `Secure`**: con OIDC activo, `SESSION_COOKIE_SECURE=false`
   SHALL abortar el arranque salvo que se declare explícitamente
   `ALLOW_INSECURE_COOKIE=true` (escape *dev-only*, sigue avisando por log).
3. **Caducidad absoluta**: nuevo `SESSION_ABSOLUTE_TTL_SECS` (12 h por defecto)
   guardado en el estado de sesión; al superarlo, la petición autenticada destruye
   la sesión y responde 401 (el `refresh` no la resucita) y `GET /auth/me`
   responde `authenticated: false`.
4. **`nbf` y `azp` del ID token**: `validation.validate_nbf = true` (con *leeway*
   pequeño) y, si el token trae `azp`, SHALL coincidir con el `client_id`.
5. **`Cache-Control: no-store`** en todas las respuestas de la API autenticadas
   por cookie o por `Authorization: Bearer` (capa única, no por endpoint).
6. **Redacción de secretos**: `Debug` manual para `Config`, `OidcConfig` y
   `SessionConfig` que sustituye `OIDC_CLIENT_SECRET` y `SESSION_SECRET` por
   `[REDACTED]`.
7. **Documentación (sin código)**: notas (3) `OIDC_ENABLED=false` por defecto y
   (12e) HSTS/`Referrer-Policy` delegados al *reverse proxy* se documentan en
   `deploy/README.md` y en la spec de `deployment`.

## Non-goals

- **Cifrado en reposo** de los tokens OIDC en `sessions.data` (nota 11): requiere
  migración, gestión de clave y rotación; va en su propio change.
- **RP-initiated logout** (nota 12d): depende del `end_session_endpoint` de
  Pocket ID; `post_logout_redirect_uri` ya está reservado en la config.
- Backlog SSRF del import (notas 1, 2, 4, 5, 6, 7): change aparte.
- Modo abierto `fail-closed` (nota 3): se documenta, no se cambia el
  comportamiento (es deliberadamente *dev-only* y ya avisa).
- Oráculo de estados 400/502/422 (nota 6): riesgo aceptado, se documenta en #81.

## Impacto de despliegue

- La cookie pasa a llamarse `__Host-oximeal_session`: las sesiones existentes
  **se invalidan** (cambio de nombre ⇒ cookie nueva). Es aceptable en un
  endurecimiento de seguridad y se anota en el `CHANGELOG` y en `deploy/README.md`.
- `__Host-` y `Secure` exigen HTTPS: en el despliegue recomendado el TLS lo
  termina el *reverse proxy* (`deploy/compose.yaml`), así que no cambia nada
  operativamente. Un despliegue en HTTP plano con OIDC dejará de arrancar salvo
  `ALLOW_INSECURE_COOKIE=true`.

## Contratos

```rust
// config.rs
pub struct SessionConfig {
    pub secret: Option<String>,
    pub ttl: Duration,              // inactividad (ya existía)
    pub absolute_ttl: Duration,     // NUEVO: vida máxima
    pub refresh_margin: Duration,
    pub cookie_name: String,        // por defecto "__Host-oximeal_session"
    pub cookie_secure: bool,
}
// NUEVO: Debug manual (redacta secretos) en Config/OidcConfig/SessionConfig
// session.rs
pub struct AuthenticatedSession { /* … */ pub started_at: i64 }  // NUEVO
// oidc.rs: validation.validate_nbf = true; comprobación de azp
```

## Proceso (SDD + TDD)

1. **RED**: tests que hoy fallan — `Debug` no filtra secretos; el nombre por
   defecto empieza por `__Host-`; `SESSION_COOKIE_SECURE=false` + OIDC ⇒ `Err`
   salvo escape; `verify_id_token` rechaza `nbf` futuro y `azp` ajeno; una sesión
   con `started_at` anterior a la vida absoluta recibe 401; las respuestas
   autenticadas llevan `no-store`.
2. **GREEN**: implementación mínima de cada punto.
3. **REFACTOR**: `fmt`, `clippy -D warnings`, suite completa ± `embed-ui`.
4. **Puertas**: `just ci`, `just ci-ui`, `openspec validate --all --strict`,
   CHANGELOG, PR.

## Rama

`feature/security-hardening-auth` desde `development` (no hay prisa: llega a
producción en la próxima release).
