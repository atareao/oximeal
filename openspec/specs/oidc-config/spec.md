# oidc-config Specification

## Purpose
Esta capacidad define la configuración OIDC por variables de entorno, el modo sin autenticación (`OIDC_ENABLED=false`), el descubrimiento y JWKS con caché, las sesiones persistentes y el IdP de desarrollo.

## Requirements

### Requirement: Configuración por entorno

El backend SHALL leer su configuración de variables de entorno:
`OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`,
`OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`,
`ALLOWED_ORIGINS`, `OIDC_ALLOWED_GROUPS` y las `SESSION_*` (secreto,
duraciones y cookie). Con OIDC activo, si falta un valor obligatorio, o
`SESSION_COOKIE_SECURE=false` no va acompañado de
`ALLOW_INSECURE_COOKIE=true`, SHALL fallar el arranque.

#### Scenario: Configuración incompleta con OIDC activo

- GIVEN `OIDC_ENABLED=true` y falta una variable obligatoria
- WHEN el servidor arranca
- THEN falla el arranque con un mensaje claro

#### Scenario: Secreto de sesión por defecto

- GIVEN `SESSION_SECRET` con un valor por defecto conocido
- WHEN el servidor arranca con OIDC activo
- THEN falla el arranque

#### Scenario: Cookie insegura rechazada

- GIVEN `OIDC_ENABLED=true` y `SESSION_COOKIE_SECURE=false` sin `ALLOW_INSECURE_COOKIE=true`
- WHEN el servidor arranca
- THEN falla el arranque con un mensaje que explica el escape *dev-only*

#### Scenario: Nombre de cookie con prefijo `__Host-`

- GIVEN el nombre de cookie por defecto o uno configurado con prefijo `__Host-`
- WHEN el servidor arranca
- THEN el nombre es válido solo si la cookie es `Secure` y usa `Path=/` sin `Domain`

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

### Requirement: Secretos fuera de los registros

La representación de depuración de la configuración SHALL redactar
`OIDC_CLIENT_SECRET` y `SESSION_SECRET` sustituyéndolos por `[REDACTED]`, de
modo que un `{:?}` accidental o un log de diagnóstico no los exponga.

#### Scenario: Depuración redactada

- GIVEN la configuración cargada con `OIDC_CLIENT_SECRET` y `SESSION_SECRET` definidos
- WHEN se formatea con `{:?}`
- THEN la salida muestra `[REDACTED]` en lugar de los valores
