## MODIFIED Requirements

### Requirement: Alta y edición desde la UI

La aplicación SHALL permitir crear y editar recetas desde la UI mediante un formulario que valide los campos requeridos y muestre los errores de validación del servidor, sin permitir guardar datos inválidos. El formulario SHALL incluir un selector de etiquetas con creación al vuelo (etiquetas existentes o nuevas, enviadas por nombre) y SHALL mostrar las etiquetas automáticas sugeridas de forma distinguible, permitiendo retirarlas —enviándolas en `suppressed_auto_tags`— y desactivar el etiquetado automático con `auto_tags`.

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
