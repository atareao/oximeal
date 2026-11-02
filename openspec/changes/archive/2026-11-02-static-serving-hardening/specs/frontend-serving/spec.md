# Capability: frontend-serving

## ADDED Requirements

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
