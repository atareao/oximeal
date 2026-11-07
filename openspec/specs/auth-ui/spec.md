# auth-ui Specification

## Purpose
Esta capacidad define el comportamiento de autenticación en el frontend: estado de sesión al arrancar, guard de rutas con redirección al login, manejo de 401/403, cabecera anti-CSRF en las mutaciones y control de identidad y cierre de sesión.

## Requirements

### Requirement: Estado de sesión en el frontend
Al arrancar, la aplicación SHALL consultar `GET /auth/me`. Mientras el estado no se conozca, SHALL mostrar un estado de carga y SHALL **no** renderizar las vistas de datos.

#### Scenario: Estado de carga inicial
- GIVEN la aplicación arrancando sin conocer todavía el estado de sesión
- WHEN la consulta a `GET /auth/me` aún no ha respondido
- THEN la aplicación muestra un estado de carga y no renderiza las vistas de datos

### Requirement: Redirección al login
Cuando no haya sesión, la aplicación SHALL redirigir a `/auth/login`, conservando el destino solicitado para volver a él tras autenticarse.

#### Scenario: Arranque sin sesión
- GIVEN una aplicación que arranca sin sesión válida
- WHEN `GET /auth/me` responde `authenticated: false`
- THEN la aplicación redirige a `/auth/login` conservando la ruta de destino

### Requirement: Manejo de 401/403
Cuando cualquier petición devuelva 401, la aplicación SHALL llevar al usuario de nuevo al login, salvo que no haya conexión y existan datos cacheados, en cuyo caso SHALL mantener la sesión en modo degradado mostrando los datos guardados con un aviso. Cuando devuelva 403, SHALL mostrar un mensaje de permisos insuficientes **sin** destruir la sesión.

#### Scenario: 401 durante el uso
- GIVEN una aplicación en uso cuya sesión caduca
- WHEN una petición devuelve 401
- THEN la aplicación redirige de nuevo al login

#### Scenario: 401 sin conexión con datos en caché
- GIVEN una sesión caducada, sin conexión y con datos en caché
- WHEN la aplicación intenta consultar datos
- THEN los datos cacheados se muestran con un aviso de modo degradado
- AND la aplicación no redirige al login

#### Scenario: 403 durante el uso
- GIVEN una sesión válida sin permiso para una acción
- WHEN una petición devuelve 403
- THEN la aplicación muestra un aviso de permisos insuficientes y mantiene la sesión intacta

### Requirement: Cabecera CSRF
El cliente del frontend SHALL añadir la cabecera `X-CSRF-Token`, con el valor obtenido de `GET /auth/me`, a todas las mutaciones autenticadas por cookie.

#### Scenario: Mutación envía CSRF
- GIVEN una sesión válida y el token anti-CSRF obtenido de `GET /auth/me`
- WHEN el frontend realiza una mutación
- THEN la petición incluye la cabecera `X-CSRF-Token`

### Requirement: Cierre de sesión en la barra superior

La aplicación SHALL ofrecer el cierre de sesión **únicamente** en la barra superior, como entrada `Salir` situada a la derecha de Configuración. La entrada `Salir` SHALL estar **siempre visible** en todos los estados de sesión. La página de ajustes SHALL **no** ofrecer ningún control de cierre de sesión. No existen roles ni administración.

#### Scenario: Salir siempre visible

- GIVEN cualquier estado de sesión (loading, authenticated, anonymous, disabled o degraded)
- WHEN se renderiza la barra superior
- THEN la entrada `Salir` está disponible a la derecha de Configuración

#### Scenario: Ajustes sin control de cierre de sesión

- GIVEN cualquier estado de sesión
- WHEN se abre `/settings`
- THEN la página de ajustes no muestra ningún botón ni control de cierre de sesión

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
