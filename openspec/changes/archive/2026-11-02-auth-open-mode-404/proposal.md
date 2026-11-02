# Change: modo abierto — las rutas `/auth/*` no caen al fallback SPA

## Why

Con `OIDC_ENABLED=false` (el valor por defecto, modo abierto documentado en
`backend/src/config.rs`) la aplicación embebida es inservible: `GET /auth/me` no
casa con ninguna ruta (las rutas `/auth/*` solo se montan con OIDC activo), cae
al **fallback SPA** y devuelve `index.html` con **200**. El frontend espera un
**404** para concluir «autenticación desactivada», así que al recibir HTML
termina en `anonymous`, redirige a `/auth/login` (que tampoco existe sin OIDC) y
todas las rutas acaban en el 404 de la SPA.

Regresión introducida al añadir el fallback SPA (`39026a3`, feature `embed-ui`),
presente ya en `main` (0.3.0) y detectada al verificar el DoD de la Fase 5 con
un smoke E2E real.

## What Changes

- El fallback del frontend clasifica `/auth` y `/auth/*` como rutas de **API**,
  de modo que nunca sirve `index.html` bajo ese prefijo.
- Con OIDC desactivado, `GET /auth/me` pasa a responder el **404 con el sobre
  JSON de la API**, que es el contrato que el frontend ya documenta y consume.
- Con OIDC activo nada cambia para las rutas reales (casan antes del fallback);
  una ruta `/auth/*` inexistente pasa a responder 404 JSON en vez de HTML.

## Impact

- Capabilities: delta en `frontend-serving` (nuevo requisito) y en `auth-session`
  (modo abierto sin OIDC).
- Backend: `backend/src/static_files.rs` (`is_api_path`), `backend/tests/`.
- Sin cambios en el frontend: ya implementa el contrato del 404.
- Retrocompatible: con OIDC activo, el comportamiento de las rutas existentes no
  se altera.

Cierra el issue #36.
