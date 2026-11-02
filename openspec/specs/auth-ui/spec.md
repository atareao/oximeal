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

### Requirement: Identidad y cierre de sesión en la interfaz
La aplicación SHALL mostrar el nombre del usuario de la sesión y SHALL ofrecer un control de cierre de sesión que invoca `POST /auth/logout`. Al completarse el cierre de sesión, la aplicación SHALL volver al login. No existen roles ni acciones de administración.

#### Scenario: Se muestra el nombre del usuario
- GIVEN una sesión válida
- WHEN la aplicación renderiza la interfaz
- THEN el nombre del usuario de la sesión es visible

#### Scenario: Cierre de sesión desde la interfaz
- GIVEN un usuario autenticado
- WHEN pulsa el control de cierre de sesión
- THEN el frontend invoca `POST /auth/logout` y vuelve al login
