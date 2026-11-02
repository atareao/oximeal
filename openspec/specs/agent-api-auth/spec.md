# agent-api-auth Specification

## Purpose
Esta capacidad define cómo se protege la Agent API (`/api/agent/*`) mediante acceso máquina-a-máquina (M2M) con `client_credentials` de un cliente confidencial de Pocket ID, exigiendo permisos dedicados `agent:*` que no reciben los clientes user-delegated.

## Requirements

### Requirement: Acceso máquina-a-máquina
Los cinco endpoints de `/api/agent/*` SHALL consumirse con un access token obtenido por `client_credentials` desde un cliente **confidencial** de Pocket ID con *Client access (M2M)* habilitado (por ejemplo, Valet). El backend SHALL exigir los permisos dedicados `agent:plan`, `agent:shopping` y `agent:read`, que **ningún** cliente user-delegated SHALL recibir. Los clientes públicos SHALL **no** poder obtener tokens M2M.

#### Scenario: Token M2M con el permiso de plan
- GIVEN un access token M2M con el permiso `agent:plan`
- WHEN se invoca `POST /api/agent/weekly-plan`
- THEN el plan semanal se escribe con normalidad

#### Scenario: Token de usuario sin permisos de agente
- GIVEN un access token de usuario sin ninguno de los permisos `agent:*`
- WHEN se invoca cualquier endpoint de `/api/agent/*`
- THEN se responde 403

#### Scenario: Cliente público intenta M2M
- GIVEN un cliente OIDC público sin *Client access (M2M)*
- WHEN intenta obtener un token por `client_credentials`
- THEN el proveedor de identidad no emite el token M2M

### Requirement: Compatibilidad del contrato
La forma de las peticiones y de las respuestas de la Agent API SHALL **no** cambiar. Este change SHALL añadir únicamente la capa de autorización sobre el contrato existente.

#### Scenario: Contrato sin cambios
- GIVEN una llamada válida a un endpoint de `/api/agent/*` con un token autorizado
- WHEN se procesa la petición
- THEN el cuerpo de la respuesta conserva exactamente el mismo esquema que antes de este change

### Requirement: Retirada del API key estático
El backend SHALL **no** aceptar un API key estático en la cabecera `Authorization` ni en variables como `OXIMEAL_API_KEY` (issue #5). Esa vía SHALL quedar obsoleta en favor de OIDC, y la migración SHALL documentarse.

#### Scenario: Petición con API key obsoleto
- GIVEN una petición que presenta el antiguo API key estático
- WHEN se invoca un endpoint de `/api/agent/*`
- THEN se responde 401 al no ser una credencial OIDC válida
