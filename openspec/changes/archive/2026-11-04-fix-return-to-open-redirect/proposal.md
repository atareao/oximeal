# Change: destino de retorno seguro tras el login (open redirect)

## Why

Auditoría de seguridad run-1, hallazgo **#80** (MEDIUM, CWE-601).

`sanitize_return_to` (`backend/src/auth/mod.rs:320`) acepta solo valores que
empiezan por `/`, que **no** empiezan por `//` y que **no** contienen `\`. No mira
los **caracteres de control**. Un `return_to` como `/%09/evil.com` pasa el filtro,
se guarda en `PreAuthFlow.return_to` (`backend/src/auth/routes.rs:116`) y, tras el
login, se emite como cabecera `Location` del 302 (`routes.rs:220,233` →
`redirect_found`, `routes.rs:79-83`). El crate `http 1.5.0` considera **válido** el
TAB en una cabecera (`value.rs:563-564`: `b == b'\t'`), y los navegadores eliminan
TAB/LF/CR al parsear la URL (WHATWG), así que `/<TAB>/evil.com` se interpreta como
`//evil.com` → el usuario acaba en `https://evil.com` justo después de
autenticarse.

Impacto: *open redirect* autenticado ⇒ **phishing** de alta credibilidad (la
redirección sale del dominio legítimo). No hay robo de tokens: el canje del código
es server-side (BFF intacto).

## What Changes

- `sanitize_return_to` SHALL rechazar cualquier **carácter de control**
  (`char::is_control`: TAB, CR, LF, …), además de las reglas vigentes
  (`/` inicial, no `//`, no `\`).
- Un `return_to` inválido se descarta y el destino de retorno SHALL ser `/`
  (comportamiento actual del `unwrap_or_else`, sin cambios).
- Se conserva el contrato con el frontend: `loginUrl()` sigue mandando una ruta
  interna codificada (`frontend/src/auth/redirect.ts:9`), sin caracteres de
  control, por lo que no se rompe el flujo legítimo.

## Non-goals

- No se implementa la lista de endurecimiento de #81 (rate limiting, prefijo de
  cookie, `Cache-Control: no-store`, caducidad absoluta, rangos SSRF). Va aparte.
- No se cambia el patrón BFF, ni la validación de `state`/`nonce`/PKCE, ni el
  formato de la cookie de sesión.
- No se añade `nbf`/`azp` al `id_token` (nota de endurecimiento, no es este bug).

## Contrato

```rust
/// Acepta solo rutas internas como destino de retorno (evita open redirect).
pub fn sanitize_return_to(value: Option<String>) -> Option<String>
```
Se mantiene la firma; cambia la aceptación: se añade el veto de caracteres de
control. Hoy ya existe el test `sanitize_return_to_rejects_external`
(`backend/src/auth/mod.rs:369-376`), que SHALL seguir pasando.

## Impact

- Capability `auth-session`: MODIFIED «Inicio de sesión BFF».
- Ficheros: `backend/src/auth/mod.rs` (guardas + tests), y test de regresión del
  login/callback si el arnés lo permite.
- Issue: #80.

## Proceso (SDD + TDD)

1. **RED**: ampliar los tests de `sanitize_return_to` con los casos que hoy fallan
   (`/%09/evil.com`, `/%0a/...`, `/%0d/...`) y un test de que un valor aceptado
   nunca puede escapar del origen. `cargo test` debe fallar SOLO en los nuevos.
2. **GREEN**: añadir el veto de caracteres de control (`char::is_control`).
3. **REFACTOR**: `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test` en verde.
4. **Puertas + cierre**: `just ci`, `openspec validate --all --strict`, archivar y
   PR.

## Rama y flujo git

La vulnerabilidad está en `v0.5.0` (producción) ⇒ rama **`hotfix/return-to-open-redirect`
desde `main`**, PR a `main` y, tras el merge, sincronizar `main` → `development`
(como en #68 y #72). Alternativa si se prefiere esperar a la próxima release:
`feature/*` desde `development` con PR a `development`.
