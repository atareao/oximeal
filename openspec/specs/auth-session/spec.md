# auth-session Specification

## Purpose
Esta capacidad define el flujo BFF de autenticación OIDC: inicio de sesión con PKCE, callback y sesión server-side con cookie `httpOnly`, consulta de sesión, cierre de sesión, renovación transparente con refresh token y protección CSRF.

## Requirements

### Requirement: Inicio de sesión BFF

El backend SHALL exponer `GET /auth/login`, que redirige al endpoint de
autorización de Pocket ID con `response_type=code`, el `scope` (`openid`,
`profile`, `groups`, `offline_access`), el `resource` del API y los permisos,
`state` y `nonce` en la sesión y PKCE S256. El `return_to` opcional SHALL ser
solo una ruta interna del propio origen: se rechaza si no empieza por `/`, si
empieza por `//` o si contiene un backslash o un carácter de control; si es
inválido, se usa `/`.

#### Scenario: Redirección a Pocket ID

- GIVEN un usuario sin sesión
- WHEN se solicita `GET /auth/login`
- THEN se responde con una redirección al endpoint de autorización que incluye `response_type=code`, `code_challenge_method=S256`, `state`, `nonce`, el `resource` del API y los permisos solicitados

#### Scenario: El navegador nunca recibe tokens

- GIVEN el flujo de autorización iniciado
- WHEN el backend redirige al proveedor de identidad
- THEN la redirección no expone `client_secret` ni tokens al navegador

#### Scenario: Se conserva un destino interno legítimo

- GIVEN un `return_to` con una ruta interna del propio origen (p. ej. `/shopping` o `/recipes?q=pasta`)
- WHEN se inicia el login y se completa el callback
- THEN el destino aceptado se conserva y la redirección final apunta a esa ruta interna
- AND el `Location` emitido es una ruta del propio origen

#### Scenario: Destino externo o con caracteres de control rechazado

- GIVEN un `return_to` que no empieza por `/`, que empieza por `//`, que contiene `\\`, o que incluye un carácter de control como TAB (`%09`), LF (`%0a`) o CR (`%0d`)
- WHEN se solicita `GET /auth/login`
- THEN el valor se descarta y el destino de retorno es `/`
- AND tras el callback el `Location` nunca apunta a un origen distinto (no hay open redirect)

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

El backend SHALL exponer `POST /auth/logout`, que destruye la sesión server-side, limpia la cookie y responde 200 con `{"logout_url": string|null}`. Con OIDC activo y `end_session_endpoint`, `logout_url` SHALL apuntar a ese endpoint con `id_token_hint` (si la sesión tiene `id_token`), `client_id` y `post_logout_redirect_uri` codificados. Sin sesión autenticada o sin `end_session_endpoint`, SHALL ser `null`. La petición SHALL seguir exigiendo `X-CSRF-Token` cuando haya sesión cookie.

#### Scenario: Logout limpia la cookie

- GIVEN una sesión válida
- WHEN se solicita `POST /auth/logout` con `X-CSRF-Token`
- THEN se responde 200, la sesión se destruye y la cookie se limpia

#### Scenario: Logout devuelve la URL de fin de sesión

- GIVEN OIDC activo, una sesión con `id_token` y un `end_session_endpoint` descubierto
- WHEN se solicita `POST /auth/logout` con `X-CSRF-Token`
- THEN la respuesta es 200 con `logout_url` apuntando al `end_session_endpoint`
- AND la URL incluye `id_token_hint`, `client_id` y `post_logout_redirect_uri` con codificación de query correcta

#### Scenario: Modo abierto sin ruta de logout

- GIVEN `OIDC_ENABLED=false`
- WHEN se solicita `POST /auth/logout`
- THEN la ruta no existe (404), porque las rutas `/auth/*` solo se montan con OIDC activo
- AND el SPA no la invoca: en estado `disabled`, `Salir` recarga la aplicación

#### Scenario: Sin end_session_endpoint

- GIVEN OIDC activo pero el IdP no publica `end_session_endpoint`
- WHEN se solicita `POST /auth/logout`
- THEN se responde 200 con `logout_url: null`

#### Scenario: Sin sesión autenticada

- GIVEN una petición sin cookie de sesión válida
- WHEN se solicita `POST /auth/logout`
- THEN se responde 200 con `logout_url: null`

#### Scenario: La URL no expone tokens en el cuerpo

- GIVEN una sesión con `id_token`
- WHEN se solicita `POST /auth/logout`
- THEN el cuerpo JSON contiene únicamente la propiedad `logout_url`
- AND no se incluye ningún token como campo del cuerpo ni se registra en logs

#### Scenario: Logout sin CSRF es rechazado

- GIVEN una sesión cookie válida
- WHEN se solicita `POST /auth/logout` sin `X-CSRF-Token`
- THEN se responde 403 y la sesión no se destruye

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
