# recipe-catalog Specification

## Purpose
Define la interfaz del catálogo de recetas de OxiMeal: la página `/recipes` con listado, búsqueda por texto y paginación, y la página de detalle `/recipes/:id` con alta, edición y borrado con confirmación. La navegación ofrece la pestaña Recetas tanto en el panel lateral (escritorio) como en la barra inferior (móvil), con objetivos táctiles de al menos 48 px y un degradado coherente sin conexión.

## Requirements

### Requirement: Página de recetas

La aplicación SHALL ofrecer una página `/recipes` que liste las recetas con
búsqueda por texto (`q`) y paginación, reutilizando `useRecipes`/`fetchRecipes`,
con estados explícitos de carga, error y lista vacía. SHALL permitir además
**filtrar por etiqueta**, combinándolo con la búsqueda y la paginación, y SHALL
reflejar el filtro activo en la URL mediante el parámetro `tag` para que el
listado sea enlazable.

#### Scenario: Listado con búsqueda y paginación

- GIVEN recetas existentes en el servidor
- WHEN el usuario abre `/recipes` y escribe un texto de búsqueda
- THEN se muestran las recetas coincidentes paginadas
- AND se refleja el estado de carga mientras llega la respuesta

#### Scenario: Filtro por etiqueta

- GIVEN etiquetas existentes y recetas asociadas a ellas
- WHEN el usuario selecciona una etiqueta en el recetario
- THEN el listado muestra solo las recetas con esa etiqueta
- AND el filtro se combina con el texto de búsqueda y la paginación

#### Scenario: Filtro reflejado en la URL

- GIVEN el recetario con un filtro de etiqueta activo
- WHEN el usuario comparte o recarga la URL
- THEN el parámetro `tag` conserva la etiqueta seleccionada
- AND el listado se reconstruye con ese filtro

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
instrucciones, tiempo de preparación, raciones, origen y **etiquetas**— al
seleccionarla, reutilizando `useRecipe`/`fetchRecipe`.

#### Scenario: Ver detalle

- GIVEN una receta en el listado
- WHEN el usuario la selecciona
- THEN se muestra su detalle completo con ingredientes, instrucciones y etiquetas
- AND se ofrecen las acciones de editar y borrar

#### Scenario: Detalle sin etiquetas

- GIVEN una receta sin etiquetas
- WHEN el usuario abre su detalle
- THEN no se muestra ninguna etiqueta
- AND el resto del detalle se presenta con normalidad

#### Scenario: Detalle no encontrado

- GIVEN un id inexistente
- WHEN el usuario accede al detalle
- THEN se muestra un estado de no encontrado
- AND se permite volver al listado

### Requirement: Alta y edición desde la UI

La aplicación SHALL permitir crear y editar recetas desde la UI con un formulario que valide los campos requeridos y muestre los errores del servidor, sin guardar datos inválidos. El formulario SHALL incluir un selector de etiquetas con creación al vuelo (por nombre) y SHALL mostrar las etiquetas automáticas sugeridas de forma distinguible, permitiendo retirarlas (en `suppressed_auto_tags`) y desactivar el etiquetado automático con `auto_tags`.

#### Scenario: Alta de receta

- GIVEN el formulario de alta abierto
- WHEN el usuario completa campos válidos y guarda
- THEN se envía la receta y el listado se actualiza con la nueva receta

#### Scenario: Edición de receta

- GIVEN el detalle de una receta existente
- WHEN el usuario edita campos y guarda
- THEN se envía `PUT` y se muestran los datos actualizados

#### Scenario: Selección y creación de etiquetas

- GIVEN el formulario de receta abierto con etiquetas existentes
- WHEN el usuario selecciona una etiqueta existente y escribe una nueva
- THEN ambas se incluyen en el payload `tags` de la receta
- AND al guardar se reflejan en la tarjeta y el detalle

#### Scenario: Retirar una sugerencia automática

- GIVEN el formulario de edición con una etiqueta automática «Sin gluten» mostrada como sugerida
- WHEN el usuario la retira y guarda
- THEN el payload incluye `sin-gluten` en `suppressed_auto_tags`
- AND tras guardar la etiqueta no reaparece

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

### Requirement: Etiquetas en el listado

La tarjeta de receta del recetario SHALL mostrar las etiquetas de la receta, distinguiendo visualmente las manuales de las automáticas sugeridas, y SHALL permitir iniciar el filtrado por una de ellas.

#### Scenario: Etiquetas en la tarjeta

- GIVEN una receta con etiquetas en el listado
- WHEN se muestra su tarjeta
- THEN se ven sus etiquetas
- AND al seleccionar una se aplica el filtro por esa etiqueta

#### Scenario: Distinción manual frente a automática

- GIVEN una receta con una etiqueta manual y una automática
- WHEN se muestra su tarjeta
- THEN ambas se ven con una presentación distinta según su `source`
- AND al seleccionar una se aplica el filtro por esa etiqueta

### Requirement: Añadir una receta a la lista de la compra

La página de detalle `/recipes/:id` SHALL ofrecer una acción «Añadir a la lista»
con un selector de raciones (por defecto las de la receta) que llame a
`POST /api/shopping-list/from-recipe` con el `recipe_id` y las raciones
elegidas, muestre el resultado y ofrezca un enlace a la lista de la compra.
SHALL invalidar la caché de la lista para que `/shopping` refleje el volcado.

#### Scenario: Añadir a la lista

- GIVEN el detalle de una receta existente
- WHEN el usuario pulsa «Añadir a la lista» y elige unas raciones
- THEN se envía la petición con `recipe_id` y `servings`
- AND se muestra una confirmación con enlace a `/shopping`

#### Scenario: Raciones por defecto

- GIVEN el detalle de una receta con `servings = 2`
- WHEN se abre el selector de raciones
- THEN el valor por defecto es 2

#### Scenario: Error del servidor

- GIVEN una petición de volcado que falla (p. ej. `404`)
- WHEN se procesa
- THEN se muestra un mensaje de error al usuario
- AND la vista del detalle permanece sin cambios

#### Scenario: Sin conexión

- GIVEN la aplicación sin conexión
- WHEN el usuario ve la acción «Añadir a la lista»
- THEN la acción está deshabilitada o en estado degradado
- AND no se promete una persistencia inexistente

#### Scenario: Accesibilidad móvil

- GIVEN la acción y el selector de raciones en un viewport móvil
- WHEN se inspeccionan
- THEN su objetivo táctil es de al menos 48 px
- AND disponen de etiquetas y roles accesibles
