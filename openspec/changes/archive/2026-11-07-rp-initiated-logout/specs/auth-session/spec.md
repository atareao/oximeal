# Capability: auth-session

## MODIFIED Requirements

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