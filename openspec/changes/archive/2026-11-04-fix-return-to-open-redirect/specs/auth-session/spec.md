# Capability: auth-session

## MODIFIED Requirements

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
