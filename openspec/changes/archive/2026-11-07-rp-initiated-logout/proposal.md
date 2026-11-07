# Proposal

## Why

Cerrar sesión en OxiMeal solo destruye la sesión local y limpia la cookie. El
proveedor OIDC (Pocket ID) conserva su sesión SSO, de modo que al volver al login
reautentica al instante y el usuario percibe que «solo se ha recargado la
página». El backend ya descubre `end_session_endpoint` y valida
`OIDC_POST_LOGOUT_REDIRECT_URI`, pero ninguno se usa; tampoco se aprovecha el
`id_token` que la sesión guarda server-side.

Se quiere un cierre de sesión completo **RP-initiated logout**: cerrar también la
sesión del IdP y volver a la aplicación con una navegación de navegador, sin
exponer tokens en el cuerpo de la respuesta ni en logs.

## What Changes

- `POST /auth/logout` destruye la sesión, limpia la cookie y responde **200**
  con `{"logout_url": string|null}` en lugar de 204.
- El backend construye `logout_url` desde el `end_session_endpoint` descubierto,
  añadiendo `id_token_hint` (el `id_token` de la sesión, si existe), `client_id`
  y `post_logout_redirect_uri` (`OIDC_POST_LOGOUT_REDIRECT_URI`), con
  codificación de query correcta.
- `logout_url` es `null` si OIDC está desactivado, si el IdP no publica
  `end_session_endpoint` o si no hay sesión autenticada.
- El frontend, tras limpiar estado y caché, navega a `logout_url` con
  `window.location.assign` (redirección de navegador, no `fetch`). Si es `null`,
  mantiene el comportamiento actual: estado anónimo y vuelta al login.
- El guard CSRF se mantiene: `POST /auth/logout` sigue exigiendo
  `X-CSRF-Token` cuando hay sesión cookie.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `auth-session`: el cierre de sesión pasa de 204 sin cuerpo a 200 con
  `logout_url`, construida desde el `end_session_endpoint` del IdP.
- `auth-ui`: `Salir` con sesión autenticada limpia estado/caché y navega a
  `logout_url` cuando existe; sin URL conserva el retorno al login.

## Impact

- Backend: `auth/routes.rs` (handler `logout`), `auth/oidc.rs` (helper de
  construcción de la URL) y tests asociados. Sin cambios de esquema ni de
  configuración: reutiliza `OIDC_POST_LOGOUT_REDIRECT_URI`.
- Frontend: `SessionProvider` (tipo y manejo de la respuesta de logout),
  `AppHeader` y sus tests. Sin cambios de rutas.
- Operación: requiere que la *Logout callback URL* del cliente en Pocket ID
  incluya `https://oximeal.territoriolinux.es/` (ya configurada).