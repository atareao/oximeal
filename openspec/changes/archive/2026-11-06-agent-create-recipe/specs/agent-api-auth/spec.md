## MODIFIED Requirements

### Requirement: Acceso máquina-a-máquina
Los seis endpoints de `/api/agent/*` SHALL consumirse con un access token obtenido por `client_credentials` desde un cliente **confidencial** de Pocket ID con *Client access (M2M)* habilitado (por ejemplo, Valet). El backend SHALL exigir los permisos dedicados `agent:read`, `agent:plan`, `agent:shopping` y `agent:write`, que **ningún** cliente user-delegated SHALL recibir. Los clientes públicos SHALL **no** poder obtener tokens M2M.

#### Scenario: Token M2M con el permiso de plan
- GIVEN un access token M2M con el permiso `agent:plan`
- WHEN se invoca `POST /api/agent/weekly-plan`
- THEN el plan semanal se escribe con normalidad

#### Scenario: Token M2M con el permiso de escritura
- GIVEN un access token M2M con el permiso `agent:write`
- WHEN se invoca `POST /api/agent/recipes`
- THEN la receta se crea con normalidad

#### Scenario: Token de usuario sin permisos de agente
- GIVEN un access token de usuario sin ninguno de los permisos `agent:*`
- WHEN se invoca cualquier endpoint de `/api/agent/*`
- THEN se responde 403

#### Scenario: Cliente público intenta M2M
- GIVEN un cliente OIDC público sin *Client access (M2M)*
- WHEN intenta obtener un token por `client_credentials`
- THEN el proveedor de identidad no emite el token M2M