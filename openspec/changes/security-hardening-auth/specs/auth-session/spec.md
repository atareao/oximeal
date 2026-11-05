# Capability: auth-session

## MODIFIED Requirements

### Requirement: Callback y sesión server-side

El backend SHALL exponer `GET /auth/callback`, que valida el `state` contra el
guardado en sesión, intercambia el código **en servidor** y valida el ID token
(firma, `iss`, `aud`, `exp`, `nbf` y `nonce`, y `azp` si viene). SHALL exigir
grupo permitido y crear una sesión nueva con cookie `httpOnly`, `Secure`,
`SameSite=Lax`, `Path=/` y nombre con prefijo `__Host-`, rotando el
identificador al autenticar. Ante error SHALL responder 400 o 403 **sin**
crear sesión.

#### Scenario: Login feliz

- GIVEN un `state` válido, un código correcto y un ID token válido de un usuario en un grupo permitido
- WHEN se solicita `GET /auth/callback`
- THEN se crea una sesión nueva, se rota el identificador y se responde con la cookie `httpOnly`, `Secure` y con prefijo `__Host-`

#### Scenario: State no coincide

- GIVEN un `state` que no coincide con el guardado en la sesión
- WHEN se solicita `GET /auth/callback`
- THEN se responde 400 y no se crea ninguna sesión

#### Scenario: ID token inválido

- GIVEN un ID token con firma, `iss`, `aud`, `exp`, `nbf` o `nonce` inválidos, o con `azp` distinto del `client_id`
- WHEN se solicita `GET /auth/callback`
- THEN se responde 400 o 403 sin crear sesión

#### Scenario: Usuario fuera de los grupos permitidos

- GIVEN un ID token válido cuyo claim `groups` no contiene ningún grupo permitido
- WHEN se solicita `GET /auth/callback`
- THEN se responde 403 y no se crea ninguna sesión

## ADDED Requirements

### Requirement: Caducidad absoluta de sesión

La sesión SHALL tener una vida máxima absoluta (`SESSION_ABSOLUTE_TTL_SECS`,
12 horas por defecto) independiente de la actividad: superada esa antigüedad,
cualquier petición autenticada SHALL destruir la sesión y obtener 401 aunque
el *refresh* siga siendo válido, y `GET /auth/me` SHALL responder
`authenticated: false`.

#### Scenario: Sesión supera la vida absoluta

- GIVEN una sesión autenticada cuya antigüedad supera `SESSION_ABSOLUTE_TTL_SECS` y con un *refresh token* todavía válido
- WHEN se realiza una petición autenticada
- THEN el backend destruye la sesión y responde 401 en vez de renovarla

#### Scenario: Consulta de sesión caducada

- GIVEN una sesión autenticada cuya antigüedad supera `SESSION_ABSOLUTE_TTL_SECS`
- WHEN se solicita `GET /auth/me`
- THEN se responde 200 con `authenticated: false`
