# Capability: auth-session

## ADDED Requirements

### Requirement: Modo abierto sin OIDC

Con `OIDC_ENABLED=false` el backend SHALL no montar las rutas `/auth/*` y SHALL
responder `404` con el sobre JSON de la API a `GET /auth/me`, de forma que el
cliente pueda distinguir «autenticación desactivada» (modo abierto) de «sesión
anónima».

#### Scenario: Consulta de sesión en modo abierto

- GIVEN el servidor arrancado con `OIDC_ENABLED=false`
- WHEN el cliente solicita `GET /auth/me`
- THEN se responde 404 con el sobre JSON de la API
- AND el cliente queda en modo abierto (`disabled`) sin redirigir al login

#### Scenario: Sesión anónima con OIDC activo

- GIVEN el servidor arrancado con OIDC activo y sin sesión
- WHEN el cliente solicita `GET /auth/me`
- THEN se responde 200 con `{ authenticated: false }`

## MODIFIED Requirements

### Requirement: Callback y sesión server-side

El backend SHALL exponer `GET /auth/callback`, que valida el `state` contra el guardado en la sesión, intercambia el código **en servidor** y valida el ID token (firma, `iss`, `aud`, `exp` y `nonce`). SHALL exigir grupo permitido y crear una sesión nueva con cookie `httpOnly` + `Secure` + `SameSite=Lax` + `Path=/`, rotando el identificador al autenticar. Ante error SHALL responder 400 o 403 **sin** crear sesión.

#### Scenario: Login feliz

- GIVEN un `state` válido, un código de autorización correcto y un ID token válido de un usuario en un grupo permitido
- WHEN se solicita `GET /auth/callback`
- THEN se crea una sesión nueva, se rota el identificador de sesión y se responde con la cookie `httpOnly`

#### Scenario: State no coincide

- GIVEN un `state` que no coincide con el guardado en la sesión
- WHEN se solicita `GET /auth/callback`
- THEN se responde 400 y no se crea ninguna sesión

#### Scenario: ID token inválido

- GIVEN un ID token con firma, `iss`, `aud`, `exp` o `nonce` inválidos
- WHEN se solicita `GET /auth/callback`
- THEN se responde 400 o 403 sin crear sesión

#### Scenario: Usuario fuera de los grupos permitidos

- GIVEN un ID token válido cuyo claim `groups` no contiene ningún grupo permitido
- WHEN se solicita `GET /auth/callback`
- THEN se responde 403 y no se crea ninguna sesión
