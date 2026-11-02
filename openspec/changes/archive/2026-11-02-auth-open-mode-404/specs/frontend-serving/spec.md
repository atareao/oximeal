# Capability: frontend-serving

## ADDED Requirements

### Requirement: Rutas de autenticación fuera del fallback SPA

El *fallback* del frontend SHALL clasificar `/auth` y `/auth/*` como rutas de API
y SHALL responder el 404 JSON de la API en lugar de `index.html` cuando ninguna
ruta de autenticación case, con independencia de que OIDC esté activo o no.

#### Scenario: Ruta de autenticación inexistente con OIDC desactivado

- GIVEN el servidor arrancado con `OIDC_ENABLED=false`
- WHEN se solicita `GET /auth/me`
- THEN se responde 404 con el sobre JSON de la API (`error` y `message`)
- AND el cuerpo no es `index.html`

#### Scenario: Ruta de autenticación inexistente con OIDC activo

- GIVEN el servidor arrancado con OIDC activo
- WHEN se solicita `GET /auth/desconocido`
- THEN se responde 404 con el sobre JSON de la API
- AND las rutas reales de `/auth/*` siguen respondiendo con normalidad

#### Scenario: Las rutas del cliente siguen cayendo al fallback

- GIVEN el frontend empaquetado
- WHEN se solicita `GET /shopping` o `GET /recetas/pasta.v2`
- THEN se sirve `index.html` con 200
