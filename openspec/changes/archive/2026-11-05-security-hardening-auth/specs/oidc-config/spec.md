# Capability: oidc-config

## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Secretos fuera de los registros

La representación de depuración de la configuración SHALL redactar
`OIDC_CLIENT_SECRET` y `SESSION_SECRET` sustituyéndolos por `[REDACTED]`, de
modo que un `{:?}` accidental o un log de diagnóstico no los exponga.

#### Scenario: Depuración redactada

- GIVEN la configuración cargada con `OIDC_CLIENT_SECRET` y `SESSION_SECRET` definidos
- WHEN se formatea con `{:?}`
- THEN la salida muestra `[REDACTED]` en lugar de los valores
