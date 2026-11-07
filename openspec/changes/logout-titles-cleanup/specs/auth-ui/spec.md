# Capability: auth-ui

## REMOVED Requirements

### Requirement: Identidad y cierre de sesión en la interfaz

## ADDED Requirements

### Requirement: Cierre de sesión en la barra superior

La aplicación SHALL ofrecer el cierre de sesión **únicamente** en la barra superior, como entrada `Salir` situada a la derecha de Configuración. La entrada `Salir` SHALL estar **siempre visible** en todos los estados de sesión. Con sesión autenticada SHALL invocar `POST /auth/logout` y volver al login. Sin sesión autenticada (estados `loading`, `disabled`, `anonymous` o `degraded`) SHALL reiniciar la aplicación con una **recarga completa**, sin invocar `POST /auth/logout`. La página de ajustes SHALL **no** ofrecer ningún control de cierre de sesión. No existen roles ni administración.

#### Scenario: Salir siempre visible

- GIVEN cualquier estado de sesión (loading, authenticated, anonymous, disabled o degraded)
- WHEN se renderiza la barra superior
- THEN la entrada `Salir` está disponible a la derecha de Configuración

#### Scenario: Cierre de sesión con sesión autenticada

- GIVEN un usuario autenticado
- WHEN pulsa `Salir`
- THEN el frontend invoca `POST /auth/logout` y vuelve al login

#### Scenario: Reinicio sin sesión

- GIVEN un estado de sesión distinto de `authenticated`
- WHEN el usuario pulsa `Salir`
- THEN la aplicación se reinicia con una recarga completa
- AND no se invoca `POST /auth/logout`

#### Scenario: Ajustes sin control de cierre de sesión

- GIVEN cualquier estado de sesión
- WHEN se abre `/settings`
- THEN la página de ajustes no muestra ningún botón ni control de cierre de sesión

### Requirement: Identidad en la página de ajustes

La aplicación SHALL mostrar el nombre del usuario en la página de ajustes **solo** cuando haya sesión autenticada, ocultando la sección de identidad en el resto de estados.

#### Scenario: Nombre en ajustes con sesión

- GIVEN una sesión autenticada
- WHEN se abre `/settings`
- THEN el nombre del usuario es visible

#### Scenario: Ajustes sin sesión sin identidad

- GIVEN un estado de sesión distinto de `authenticated`
- WHEN se abre `/settings`
- THEN la sección de identidad no se muestra
