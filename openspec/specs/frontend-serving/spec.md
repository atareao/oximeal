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

### Requirement: Revalidación con ETag
El backend SHALL incluir una cabecera `ETag` fuerte en toda respuesta estática servida con la feature `embed-ui`, derivada del hash SHA-256 del contenido embebido, y SHALL responder `304 Not Modified` sin cuerpo cuando la petición incluya `If-None-Match` cuyo valor coincida con el `ETag`; en cualquier otro caso SHALL servir `200` con el cuerpo completo.

#### Scenario: Primera respuesta incluye ETag
- GIVEN un binario con la feature `embed-ui`
- WHEN se solicita `GET /`
- THEN la respuesta incluye una cabecera `ETag` no vacía

#### Scenario: Revalidación coincidente devuelve 304
- GIVEN el `ETag` obtenido de una respuesta previa
- WHEN se repite la petición con `If-None-Match: <ETag>`
- THEN se devuelve `304 Not Modified`
- AND la respuesta no lleva cuerpo

#### Scenario: ETag distinto devuelve 200
- GIVEN una petición con `If-None-Match: "no-coincide"`
- WHEN se solicita `GET /`
- THEN se devuelve `200` con el cuerpo completo

#### Scenario: Comodín coincide
- GIVEN una petición con `If-None-Match: *`
- WHEN se solicita `GET /`
- THEN se devuelve `304 Not Modified`

### Requirement: Charset UTF-8 en tipos textuales
El backend SHALL añadir `; charset=utf-8` a los tipos MIME textuales (`text/*`) al servir activos estáticos.

#### Scenario: HTML con charset
- GIVEN un activo `index.html` servido
- WHEN se inspecciona su `Content-Type`
- THEN el valor es `text/html; charset=utf-8`

### Requirement: Cabeceras de seguridad
El backend SHALL incluir `X-Content-Type-Options: nosniff` en todas las respuestas estáticas y SHALL incluir `Content-Security-Policy` en las respuestas de documentos HTML, con una política compatible con el frontend (Ant Design CSS-in-JS exige `style-src 'unsafe-inline'`).

#### Scenario: nosniff en todo activo
- GIVEN cualquier activo estático servido
- WHEN se inspeccionan sus cabeceras
- THEN incluye `X-Content-Type-Options: nosniff`

#### Scenario: CSP en el documento HTML
- GIVEN el `index.html` servido
- WHEN se inspeccionan sus cabeceras
- THEN incluye `Content-Security-Policy` con `script-src 'self'`

#### Scenario: Sin CSP en activos no-HTML
- GIVEN un activo `.js` servido
- WHEN se inspeccionan sus cabeceras
- THEN no incluye `Content-Security-Policy`

### Requirement: Sin soporte de Range
El backend SHALL NOT anunciar `Accept-Ranges` y SHALL servir `200` con el cuerpo completo ante una petición con `Range`.

#### Scenario: Petición con Range se sirve completa
- GIVEN un activo estático
- WHEN se solicita con `Range: bytes=0-3`
- THEN se devuelve `200` con el cuerpo completo
- AND no se incluye la cabecera `Accept-Ranges`

### Requirement: Tipos MIME de la PWA
El backend SHALL servir el manifiesto web con `Content-Type: application/manifest+json` y el Service Worker con `Cache-Control: no-cache`, con independencia de lo que detecte por defecto la biblioteca de tipos MIME.

#### Scenario: MIME del manifiesto
- GIVEN un manifiesto `manifest.webmanifest` empaquetado
- WHEN se solicita al backend
- THEN la respuesta incluye `Content-Type: application/manifest+json`

#### Scenario: Cache-Control del Service Worker
- GIVEN el fichero `sw.js` empaquetado
- WHEN se solicita al backend
- THEN la respuesta incluye `Cache-Control: no-cache`
