# Capability: oidc-config

## ADDED Requirements

### Requirement: Configuración por entorno
El backend SHALL leer su configuración OIDC de variables de entorno: `OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE` (el resource del API), `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `ALLOWED_ORIGINS`, `OIDC_ALLOWED_GROUPS` (grupos autorizados) y las variables `SESSION_*` (clave y duración). Si `OIDC_ENABLED=true` y falta algún valor obligatorio, el servidor SHALL **fallar el arranque** con un mensaje claro.

#### Scenario: Configuración incompleta con OIDC activo
- GIVEN `OIDC_ENABLED=true` y alguna variable obligatoria ausente
- WHEN el servidor arranca
- THEN el arranque falla con un mensaje que indica la configuración faltante

#### Scenario: Secreto de sesión por defecto
- GIVEN una clave de firma de sesión con el valor por defecto conocido o ausente
- WHEN el servidor arranca
- THEN el arranque falla

### Requirement: Modo sin autenticación
Con `OIDC_ENABLED=false` el servidor SHALL comportarse como antes de este change (API abierta), modo pensado **solo** para desarrollo y pruebas y para no romper `cargo test`. Al arrancar en ese modo SHALL emitir un warning explícito.

#### Scenario: OIDC desactivado
- GIVEN `OIDC_ENABLED=false`
- WHEN el servidor arranca
- THEN emite un warning y la API permanece abierta

### Requirement: Descubrimiento y JWKS
El backend SHALL usar el documento de descubrimiento OIDC y el JWKS publicados por Pocket ID, con caché y refresco. Si el proveedor de identidad no está disponible al arrancar, el servidor SHALL **arrancar igualmente** y reintentar, sin caerse por culpa del IdP.

#### Scenario: IdP caído al arrancar
- GIVEN que el proveedor de identidad no responde al arrancar
- WHEN el servidor arranca
- THEN el servidor arranca igualmente; `GET /api/health` responde 200 y las rutas protegidas sin credencial válida responden 401

#### Scenario: Refresco de JWKS
- GIVEN un JWKS cacheado y un token con un `kid` nuevo
- WHEN se valida el token
- THEN el backend refresca el JWKS antes de decidir

### Requirement: Sesiones
La sesión SHALL ser server-side y SHALL usar un almacén persistente en la base de datos SQLite del propio proyecto, con caducidad configurable. El secreto de firma de la cookie SHALL ser obligatorio y SHALL **no** tener un valor por defecto conocido.

#### Scenario: Persistencia de sesión
- GIVEN un servidor reiniciado con sesiones previamente creadas
- WHEN un usuario presenta su cookie de sesión vigente
- THEN el backend recupera la sesión desde el almacén persistente

### Requirement: IdP de desarrollo
El proyecto SHALL documentar y proporcionar un `compose` (podman/docker) con Pocket ID y su base de datos para desarrollo local, incluyendo los pasos para crear el API, los permisos, el cliente user-delegated y el cliente M2M.

#### Scenario: Levantar el IdP de desarrollo
- GIVEN el fichero `compose` de desarrollo
- WHEN se levanta con podman o docker
- THEN Pocket ID queda disponible para autenticar el flujo local documentado
