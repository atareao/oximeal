# Capability: deployment

## MODIFIED Requirements

### Requirement: Configuración por entorno

La imagen SHALL ser configurable por variables de entorno sin recompilación,
respetando `.env.example`. SHALL documentarse el **modo abierto**
(`OIDC_ENABLED=false`, solo desarrollo), el **modo OIDC**, el endurecimiento
de sesión (`__Host-`, `SESSION_COOKIE_SECURE`, `SESSION_ABSOLUTE_TTL_SECS`,
`ALLOW_INSECURE_COOKIE`) y que HSTS y `Referrer-Policy` los aporta el *reverse
proxy* que termina el TLS.

#### Scenario: Variables de entorno reconocidas

- GIVEN el fichero `.env.example`
- WHEN se configura el contenedor
- THEN se reconocen `DATABASE_URL`, `BIND_ADDR`, `RUST_LOG`, el bloque OIDC (`OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `OIDC_ALLOWED_GROUPS`), `ALLOWED_ORIGINS`, `IMPORT_ALLOWED_HOSTS` y `SESSION_*`
- AND ninguna de ellas exige recompilar la imagen

#### Scenario: Modo abierto

- GIVEN un contenedor con `OIDC_ENABLED=false`
- WHEN la aplicación arranca sin variables OIDC
- THEN la API queda accesible sin autenticación y lo advierte en logs
- AND el contenedor responde en `/api/health`

#### Scenario: Modo OIDC con configuración incompleta

- GIVEN un contenedor con `OIDC_ENABLED=true` y falta una variable OIDC obligatoria
- WHEN la aplicación arranca
- THEN el proceso falla al arrancar con un error explícito
- AND el healthcheck permanece en fallo (el contenedor no se marca `healthy`)

#### Scenario: Terminación TLS en el proxy

- GIVEN el despliegue recomendado con el TLS terminado en el *reverse proxy*
- WHEN se documenta en `deploy/README.md`
- THEN se indica que la cookie viaja con `Secure` y prefijo `__Host-`
- AND se documenta que HSTS y `Referrer-Policy` se configuran en el proxy, no en la aplicación
