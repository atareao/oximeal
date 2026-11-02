# frontend-serving Specification

## Purpose
Especifica cómo el backend sirve el frontend compilado desde `frontend/dist` con `rust-embed` cuando la feature de Cargo `embed-ui` está activa, con tipos MIME correctos y cabeceras de cache de larga duración para assets con hash. Incluye el fallback SPA para rutas no-API desconocidas sin capturar `/api/*`, y el modo degradado que arranca y responde 404 con una pista cuando la UI no está empaquetada.

## Requirements

### Requirement: Servido embebido
El backend SHALL servir el frontend compilado desde `frontend/dist` mediante `rust-embed` cuando la feature de Cargo `embed-ui` esté activa, con tipos MIME correctos y cabeceras de cache para assets con hash.

#### Scenario: Servir el índice
- GIVEN un binario compilado con la feature `embed-ui` y `frontend/dist` presente
- WHEN se solicita `GET /`
- THEN se devuelve `index.html` con `Content-Type: text/html`

#### Scenario: Asset con hash
- GIVEN un asset con hash en el nombre dentro de `frontend/dist`
- WHEN se solicita dicho asset
- THEN se devuelve con su tipo MIME correcto
- AND se incluye una cabecera de cache de larga duración

### Requirement: Fallback SPA
Las rutas que no empiecen por `/api` y que no correspondan a un asset existente SHALL devolver `index.html`. Las rutas bajo `/api/*` SHALL nunca caer al fallback SPA.

#### Scenario: Ruta SPA desconocida
- GIVEN una ruta de cliente como `/menu`
- WHEN se solicita al servidor
- THEN se devuelve `index.html`

#### Scenario: Ruta de API inexistente
- GIVEN una ruta bajo `/api` que no existe
- WHEN se solicita al servidor
- THEN se devuelve una respuesta de error de API
- AND no se devuelve `index.html`

### Requirement: Modo sin UI empaquetada
Con la feature `embed-ui` desactivada, el servidor SHALL arrancar y responder 404 con una pista en las rutas no-API, sin romper `cargo test`.

#### Scenario: Build sin dist
- GIVEN un build sin la feature `embed-ui` o sin `frontend/dist`
- WHEN se ejecuta `cargo test`
- THEN los tests pasan
- AND una petición a una ruta no-API devuelve 404 con una pista sobre la UI no empaquetada

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
