## MODIFIED Requirements

### Requirement: Identidad y cierre de sesión en la interfaz

La aplicación SHALL mostrar el nombre del usuario en la página de ajustes y SHALL ofrecer el cierre de sesión en la barra superior (entrada Salir) y en la página de ajustes, invocando `POST /auth/logout`. Al completarse, SHALL volver al login. No existen roles ni acciones de administración.

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
- THEN la entrada Salir está disponible

#### Scenario: Nombre en ajustes

- GIVEN una sesión válida
- WHEN se abre `/settings`
- THEN el nombre del usuario es visible

#### Scenario: Cierre de sesión

- GIVEN un usuario autenticado
- WHEN pulsa Salir
- THEN el frontend invoca `POST /auth/logout` y vuelve al login
