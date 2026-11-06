# api-authorization Specification

## Purpose
Esta capacidad define la validación de access tokens OIDC (RFC 9068) contra el JWKS de Pocket ID y la autorización por permisos por endpoint, tanto por cookie de sesión (SPA) como por `Authorization: Bearer`, además del CORS restringido y los endpoints públicos.

## Requirements

### Requirement: Validación de access tokens
El backend SHALL validar los access tokens presentados contra el API comprobando la firma con las claves publicadas en el JWKS de Pocket ID (con caché y refresco ante un `kid` desconocido), el `iss` esperado, que el `aud` contenga el resource del API, la vigencia `exp`/`nbf` y un algoritmo de firma permitido. SHALL rechazar los tokens con `alg=none` o con algoritmos no permitidos.

#### Scenario: Clave de firma desconocida
- GIVEN un token firmado con un `kid` ausente en el JWKS cacheado
- WHEN el backend intenta validarlo
- THEN refresca el JWKS y, si el `kid` sigue siendo desconocido, responde 401

#### Scenario: Token caducado
- GIVEN un access token con `exp` en el pasado
- WHEN el backend intenta validarlo
- THEN responde 401

#### Scenario: Audiencia de otro API
- GIVEN un access token cuyo `aud` no contiene el resource del API de OxiMeal
- WHEN el backend intenta validarlo
- THEN responde 401

#### Scenario: Algoritmo no permitido
- GIVEN un token con `alg=none` o con un algoritmo fuera de la lista permitida
- WHEN el backend intenta validarlo
- THEN responde 401

### Requirement: Autorización por permiso
Cada endpoint protegido SHALL declarar el permiso que exige (`read:recipes`, `write:recipes`, `read:plan`, `write:plan`, `read:shopping`, `write:shopping`). Cuando la credencial es válida pero no trae el permiso exigido, el backend SHALL responder 403; cuando no hay credencial válida, SHALL responder 401 e incluir la cabecera `WWW-Authenticate: Bearer`.

#### Scenario: Sin credenciales
- GIVEN una petición sin cookie de sesión ni cabecera `Authorization`
- WHEN se solicita un endpoint protegido
- THEN se responde 401 con `WWW-Authenticate: Bearer`

#### Scenario: Token sin el permiso exigido
- GIVEN un token válido cuyo claim `scope` no incluye el permiso que exige el endpoint
- WHEN se solicita el endpoint
- THEN se responde 403

#### Scenario: Token con el permiso exigido
- GIVEN un token válido cuyo claim `scope` incluye el permiso que exige el endpoint
- WHEN se solicita el endpoint
- THEN se atiende la petición con 200

### Requirement: Endpoints públicos
El backend SHALL mantener accesibles **sin** autenticación `GET /api/health`, el documento OpenAPI y la Swagger UI. El resto de rutas SHALL exigir credenciales.

#### Scenario: Health y OpenAPI sin token
- GIVEN una petición sin credenciales
- WHEN se solicitan `GET /api/health`, el documento OpenAPI o la Swagger UI
- THEN se responde 200 sin exigir autenticación

### Requirement: CORS restringido
El backend SHALL sustituir el CORS permisivo por una lista de orígenes permitidos configurable. Al autenticar por cookie de sesión, el backend SHALL **no** combinar `allow_origin(Any)` con credenciales.

#### Scenario: Origen permitido
- GIVEN un origen incluido en la lista configurada
- WHEN se realiza una petición con credenciales
- THEN la respuesta incluye las cabeceras CORS correspondientes a ese origen

#### Scenario: Origen no permitido
- GIVEN un origen que no está en la lista configurada
- WHEN se realiza una petición con credenciales
- THEN la respuesta no autoriza a ese origen

### Requirement: Sesión o Bearer
Una misma petición SHALL poder autorizarse con la cookie de sesión (SPA) o con `Authorization: Bearer <access-token>` (Valet u otros clientes), aplicando la misma tabla de permisos.

#### Scenario: Autorización por cookie de sesión
- GIVEN una sesión válida cuyos permisos incluyen el permiso exigido
- WHEN se solicita el endpoint con la cookie de sesión
- THEN se atiende la petición con 200

#### Scenario: Autorización por Bearer
- GIVEN un access token válido que incluye el permiso exigido
- WHEN se solicita el endpoint con `Authorization: Bearer`
- THEN se atiende la petición con 200 sin depender de la cookie de sesión

### Requirement: Respuestas autenticadas no cacheables

Las respuestas de la API autorizadas por cookie de sesión o por
`Authorization: Bearer` SHALL incluir `Cache-Control: no-store`, de modo que
ni el navegador ni un intermediario reutilicen contenido de un usuario para
otro.

#### Scenario: Respuesta por cookie de sesión

- GIVEN una petición a la API autorizada por cookie de sesión
- WHEN el backend responde
- THEN la respuesta incluye `Cache-Control: no-store`

#### Scenario: Respuesta por Bearer

- GIVEN una petición autorizada por `Authorization: Bearer`
- WHEN el backend responde
- THEN la respuesta incluye `Cache-Control: no-store`
