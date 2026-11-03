# Capability: recipe-catalog

## MODIFIED Requirements

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

La aplicación SHALL permitir crear y editar recetas desde la UI mediante un
formulario que valide los campos requeridos y muestre los errores de validación
del servidor, sin permitir guardar datos inválidos. El formulario SHALL incluir un
**selector de etiquetas con creación al vuelo**: el usuario puede elegir
etiquetas existentes o escribir una nueva, y las etiquetas se envían por nombre.

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

#### Scenario: Validación fallida

- GIVEN un formulario con campos obligatorios vacíos o categoría no permitida
- WHEN el usuario intenta guardar
- THEN se muestran los errores y no se envía la petición
- AND si el servidor responde 400 se muestran los errores devueltos

## ADDED Requirements

### Requirement: Etiquetas en el listado

La tarjeta de receta del recetario SHALL mostrar las etiquetas de la receta y
SHALL permitir iniciar el filtrado por una de ellas.

#### Scenario: Etiquetas en la tarjeta

- GIVEN una receta con etiquetas en el listado
- WHEN se muestra su tarjeta
- THEN se ven sus etiquetas
- AND al seleccionar una se aplica el filtro por esa etiqueta
