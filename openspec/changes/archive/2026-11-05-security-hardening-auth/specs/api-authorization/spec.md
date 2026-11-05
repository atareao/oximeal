# Capability: api-authorization

## ADDED Requirements

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
