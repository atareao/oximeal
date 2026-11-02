# auth-session Specification

## Purpose
Esta capacidad define el flujo BFF de autenticación OIDC: inicio de sesión con PKCE, callback y sesión server-side con cookie `httpOnly`, consulta de sesión, cierre de sesión, renovación transparente con refresh token y protección CSRF.

## Requirements

### Requirement: Inicio de sesión BFF
El backend SHALL exponer `GET /auth/login`, que redirige al endpoint de autorización de Pocket ID con `response_type=code`, un `scope` que incluya `openid`, `profile`, `groups`, `offline_access`, el `resource` del API y los permisos solicitados. La petición SHALL incluir un `state` y un `nonce` aleatorios guardados en la sesión, y SHALL usar PKCE con `code_challenge` y `code_challenge_method=S256`.

#### Scenario: Redirección a Pocket ID
- GIVEN un usuario sin sesión
- WHEN se solicita `GET /auth/login`
- THEN se responde con una redirección al endpoint de autorización que incluye `response_type=code`, `code_challenge_method=S256`, `state`, `nonce`, el `resource` del API y los permisos solicitados

#### Scenario: El navegador nunca recibe tokens
- GIVEN el flujo de autorización iniciado
- WHEN el backend redirige al proveedor de identidad
- THEN la redirección no expone `client_secret` ni tokens al navegador

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

### Requirement: Consulta de sesión
El backend SHALL exponer `GET /auth/me`, que devuelve 200 con `{ authenticated, subject, name, permissions, csrf_token }` cuando hay sesión, y 200 con `{ authenticated: false }` cuando no la hay. La respuesta SHALL incluir el token anti-CSRF para las mutaciones y **nunca** SHALL exponer access tokens, refresh tokens ni el ID token.

#### Scenario: Con sesión activa
- GIVEN una sesión válida de un usuario autenticado
- WHEN se solicita `GET /auth/me`
- THEN se responde 200 con `authenticated: true`, el sujeto, el nombre, los permisos y el token anti-CSRF

#### Scenario: Sin sesión
- GIVEN una petición sin cookie de sesión válida
- WHEN se solicita `GET /auth/me`
- THEN se responde 200 con `authenticated: false`

### Requirement: Cierre de sesión
El backend SHALL exponer `POST /auth/logout`, que destruye la sesión server-side, limpia la cookie y responde 204.

#### Scenario: Logout limpia la cookie
- GIVEN una sesión válida
- WHEN se solicita `POST /auth/logout`
- THEN se responde 204, la sesión se destruye y la cookie se limpia

### Requirement: Renovación transparente
El backend SHALL renovar el access token con el refresh token cuando esté a punto de expirar, según un margen configurable. Si la renovación falla o el usuario ya no pertenece a un grupo permitido, SHALL destruir la sesión y responder 401.

#### Scenario: Renovación con refresh token
- GIVEN una sesión cuyo access token está dentro del margen de expiración
- WHEN se realiza una petición autenticada
- THEN el backend obtiene un nuevo access token con el refresh token y la petición se atiende con el token renovado

#### Scenario: Refresh fallido
- GIVEN una sesión cuyo refresh token ha caducado o es rechazado
- WHEN se realiza una petición autenticada
- THEN se destruye la sesión y se responde 401

#### Scenario: Usuario ya no autorizado
- GIVEN un refresh válido cuyo usuario ya no pertenece a ningún grupo permitido
- WHEN se intenta renovar la sesión
- THEN se destruye la sesión y se responde 401

### Requirement: Protección CSRF
Las peticiones mutantes autenticadas por cookie SHALL exigir la cabecera anti-CSRF `X-CSRF-Token` cuyo valor entrega `GET /auth/me`. Las peticiones mutantes por cookie que no presenten una cabecera válida SHALL recibir 403.

#### Scenario: Mutación sin cabecera CSRF
- GIVEN una sesión válida y una petición mutante autenticada por cookie sin `X-CSRF-Token`
- WHEN se envía la petición
- THEN se responde 403 y la sesión permanece intacta

#### Scenario: Mutación con cabecera CSRF válida
- GIVEN una sesión válida y el token anti-CSRF entregado por `GET /auth/me`
- WHEN se envía una petición mutante con `X-CSRF-Token` correcto
- THEN la petición se procesa con normalidad

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
