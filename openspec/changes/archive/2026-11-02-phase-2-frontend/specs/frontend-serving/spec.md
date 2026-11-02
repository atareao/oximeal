# Capability: frontend-serving

## ADDED Requirements

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
