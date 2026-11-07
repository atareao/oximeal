# Capability: auth-ui

## MODIFIED Requirements

### Requirement: Comportamiento de `Salir` según el estado de sesión

Con sesión autenticada, `Salir` SHALL invocar `POST /auth/logout`, limpiar el estado y la caché local y, si la respuesta trae `logout_url`, navegar a esa URL con una redirección de navegador (`window.location.assign`). Si `logout_url` es `null`, SHALL quedar anónimo y volver al login. Sin sesión autenticada, `Salir` SHALL reiniciar la aplicación con una recarga completa, sin invocar `POST /auth/logout`.

#### Scenario: Cierre de sesión con sesión autenticada

- GIVEN un usuario autenticado
- WHEN pulsa `Salir`
- THEN el frontend invoca `POST /auth/logout` con `X-CSRF-Token`
- AND limpia el estado y la caché local

#### Scenario: Cierre de sesión con sesión autenticada y URL de fin de sesión

- GIVEN un usuario autenticado y el backend devuelve `logout_url`
- WHEN pulsa `Salir`
- THEN el frontend invoca `POST /auth/logout` con `X-CSRF-Token`
- AND limpia el estado y la caché local
- AND navega a `logout_url` con `window.location.assign`

#### Scenario: Cierre de sesión sin URL de fin de sesión

- GIVEN un usuario autenticado y el backend devuelve `logout_url: null`
- WHEN pulsa `Salir`
- THEN el frontend invoca `POST /auth/logout`
- AND limpia el estado y la caché local
- AND queda en estado anónimo y vuelve al login, sin navegación externa

#### Scenario: Reinicio sin sesión

- GIVEN un estado de sesión distinto de `authenticated`
- WHEN el usuario pulsa `Salir`
- THEN la aplicación se reinicia con una recarga completa
- AND no se invoca `POST /auth/logout`