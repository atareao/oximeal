# recipe-catalog Specification

## Purpose
Define la interfaz del catálogo de recetas de OxiMeal: la página `/recipes` con listado, búsqueda por texto y paginación, y la página de detalle `/recipes/:id` con alta, edición y borrado con confirmación. La navegación ofrece la pestaña Recetas tanto en el panel lateral (escritorio) como en la barra inferior (móvil), con objetivos táctiles de al menos 48 px y un degradado coherente sin conexión.

## Requirements

### Requirement: Página de recetas

La aplicación SHALL ofrecer una página `/recipes` que liste las recetas con
búsqueda por texto (`q`) y paginación, reutilizando `useRecipes`/`fetchRecipes`,
con estados explícitos de carga, error y lista vacía.

#### Scenario: Listado con búsqueda y paginación

- GIVEN recetas existentes en el servidor
- WHEN el usuario abre `/recipes` y escribe un texto de búsqueda
- THEN se muestran las recetas coincidentes paginadas
- AND se refleja el estado de carga mientras llega la respuesta

#### Scenario: Lista vacía

- GIVEN una consulta sin resultados
- WHEN la página recibe la respuesta
- THEN se muestra un estado vacío claro
- AND no se lanza ningún error al usuario

#### Scenario: Error de red

- GIVEN un fallo al cargar el listado
- WHEN la petición falla
- THEN se muestra un mensaje de error con opción de reintentar
- AND la interfaz no queda bloqueada

### Requirement: Detalle de receta

La aplicación SHALL mostrar el detalle de una receta —ingredientes,
instrucciones, tiempo de preparación, raciones y origen— al seleccionarla,
reutilizando `useRecipe`/`fetchRecipe`.

#### Scenario: Ver detalle

- GIVEN una receta en el listado
- WHEN el usuario la selecciona
- THEN se muestra su detalle completo con ingredientes e instrucciones
- AND se ofrecen las acciones de editar y borrar

#### Scenario: Detalle no encontrado

- GIVEN un id inexistente
- WHEN el usuario accede al detalle
- THEN se muestra un estado de no encontrado
- AND se permite volver al listado

### Requirement: Alta y edición desde la UI

La aplicación SHALL permitir crear y editar recetas desde la UI mediante un
formulario que valide los campos requeridos y muestre los errores de validación
del servidor, sin permitir guardar datos inválidos.

#### Scenario: Alta de receta

- GIVEN el formulario de alta abierto
- WHEN el usuario completa campos válidos y guarda
- THEN se envía la receta y el listado se actualiza con la nueva receta

#### Scenario: Edición de receta

- GIVEN el detalle de una receta existente
- WHEN el usuario edita campos y guarda
- THEN se envía `PUT` y se muestran los datos actualizados

#### Scenario: Validación fallida

- GIVEN un formulario con campos obligatorios vacíos o categoría no permitida
- WHEN el usuario intenta guardar
- THEN se muestran los errores y no se envía la petición
- AND si el servidor responde 400 se muestran los errores devueltos

### Requirement: Borrado con confirmación

La aplicación SHALL exigir una confirmación explícita antes de borrar una receta
y SHALL actualizar la vista tras un borrado correcto.

#### Scenario: Confirmar borrado

- GIVEN el detalle de una receta
- WHEN el usuario pulsa borrar y confirma
- THEN se envía `DELETE` y la receta desaparece del listado

#### Scenario: Cancelar borrado

- GIVEN el diálogo de confirmación abierto
- WHEN el usuario cancela
- THEN no se envía ninguna petición
- AND la receta permanece intacta

### Requirement: Accesibilidad móvil

Los controles interactivos de la página de recetas SHALL tener una altura mínima
de 48px y SHALL exponer etiquetas y roles accesibles para acciones, búsqueda y
formularios. En modo offline la UI SHALL degradar de forma coherente con
`offline-data`/`offline-writes`: deshabilitar las escrituras o mostrar un estado
degradado, sin prometer persistencia que no exista.

#### Scenario: Objetivos táctiles

- GIVEN la página de recetas en un viewport móvil
- WHEN se inspecciona un control interactivo
- THEN su altura es de al menos 48px

#### Scenario: Acciones accesibles

- GIVEN el listado o el detalle de una receta
- WHEN se inspeccionan las acciones y el formulario
- THEN disponen de etiquetas y roles accesibles

#### Scenario: Modo offline

- GIVEN la aplicación sin conexión
- WHEN el usuario intenta crear, editar o borrar una receta
- THEN la UI muestra el estado degradado o deshabilita la acción según el patrón de la app
- AND no se promete una persistencia inexistente
