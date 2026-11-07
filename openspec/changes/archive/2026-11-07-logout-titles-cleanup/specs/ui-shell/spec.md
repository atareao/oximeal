# Capability: ui-shell

## REMOVED Requirements

### Requirement: Página de ajustes

## ADDED Requirements

### Requirement: Página de ajustes de la aplicación

La aplicación SHALL exponer la ruta `/settings` accesible desde la entrada Configuración de la barra superior. La página SHALL contener el selector de tema y la preferencia de comensales habituales. La página SHALL **no** ofrecer ningún control de cierre de sesión; la identidad y el cierre de sesión se rigen por la capacidad `auth-ui`.

#### Scenario: Acceso a ajustes

- GIVEN la barra superior
- WHEN el usuario pulsa Configuración
- THEN navega a `/settings`

#### Scenario: Cambio de tema en ajustes

- GIVEN la página de ajustes
- WHEN el usuario cambia el tema
- THEN la preferencia se guarda y se aplica de inmediato

#### Scenario: Comensales habituales en ajustes

- GIVEN la página de ajustes
- WHEN el usuario cambia los comensales habituales y guarda
- THEN la preferencia se persiste en el servidor

### Requirement: Encabezados de vista accesibles

Cada vista de datos (Menú, Recetas, Lista de la compra y Configuración) SHALL exponer exactamente un encabezado de página que nombre la vista, **oculto visualmente** pero accesible para lectores de pantalla y referenciado por `aria-labelledby`. Las vistas SHALL **no** mostrar un título de página visible; el nombre de la vista se comunica mediante la barra de navegación y el encabezado programático.

#### Scenario: Título oculto accesible

- GIVEN cualquier vista de datos
- WHEN se renderiza la vista
- THEN existe un encabezado accesible con el nombre de la vista
- AND dicho encabezado no se muestra visualmente en pantalla

#### Scenario: Navegación como referencia visible

- GIVEN una vista de datos
- WHEN se renderiza
- THEN la barra superior marca la entrada activa con `aria-current="page"`
- AND no existe un título de página visible duplicando esa entrada
