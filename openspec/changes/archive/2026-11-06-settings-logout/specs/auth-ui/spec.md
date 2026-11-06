# Capability: auth-ui

## MODIFIED Requirements

### Requirement: Identidad y cierre de sesión en la interfaz

La aplicación SHALL mostrar el nombre del usuario en la página de ajustes. El
cierre de sesión SHALL ofrecerse **únicamente** en la barra superior, como
entrada `Salir` situada a la derecha de la entrada Configuración, e invocará
`POST /auth/logout`. Al completarse, SHALL volver al login. La página de ajustes
SHALL **no** ofrecer ningún control de cierre de sesión. No existen roles ni
acciones de administración.

#### Scenario: Se muestra el nombre del usuario

- GIVEN una sesión válida
- WHEN la aplicación renderiza la interfaz
- THEN el nombre del usuario de la sesión es visible

#### Scenario: Cierre de sesión desde la interfaz

- GIVEN un usuario autenticado
- WHEN pulsa el control de cierre de sesión
- THEN el frontend invoca `POST /auth/logout` y vuelve al login

#### Scenario: Salir desde la barra superior

- GIVEN una sesión válida
- WHEN se renderiza la barra superior
- THEN la entrada Salir está disponible a la derecha de Configuración

#### Scenario: Nombre en ajustes

- GIVEN una sesión válida
- WHEN se abre `/settings`
- THEN el nombre del usuario es visible

#### Scenario: Cierre de sesión

- GIVEN un usuario autenticado
- WHEN pulsa Salir
- THEN el frontend invoca `POST /auth/logout` y vuelve al login

#### Scenario: Ajustes sin control de cierre de sesión

- GIVEN un usuario autenticado
- WHEN se abre `/settings`
- THEN la página de ajustes no muestra ningún botón ni control de cierre de sesión
