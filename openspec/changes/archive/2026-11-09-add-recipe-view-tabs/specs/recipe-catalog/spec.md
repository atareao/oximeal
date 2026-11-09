# Spec Delta

## MODIFIED Requirements

### Requirement: Detalle de receta

La aplicación SHALL mostrar el detalle de una receta —ingredientes,
instrucciones, tiempo de preparación, raciones, origen y **etiquetas**— al
seleccionarla, reutilizando `useRecipe`/`fetchRecipe`. Las secciones de
instrucciones e ingredientes SHALL organizarse en dos pestañas accesibles:
**Instrucciones** (activa por defecto) e **Ingredientes**, y el resto del
detalle SHALL permanecer visible por encima de ellas.

#### Scenario: Ver detalle

- GIVEN una receta en el listado
- WHEN el usuario la selecciona
- THEN se muestra su detalle con las pestañas Instrucciones e Ingredientes
- AND la pestaña Instrucciones está activa por defecto
- AND se ofrecen las acciones de editar y borrar

#### Scenario: Pestaña por defecto

- GIVEN una receta con instrucciones
- WHEN el usuario abre su detalle
- THEN el panel de Instrucciones es visible y el de Ingredientes no
- AND la pestaña Instrucciones aparece seleccionada

#### Scenario: Cambiar a Ingredientes

- GIVEN el detalle de una receta con ingredientes
- WHEN el usuario selecciona la pestaña Ingredientes
- THEN se muestra la lista de ingredientes con nombres y cantidades (escaladas si procede)
- AND el panel de Instrucciones deja de mostrarse

#### Scenario: Volver a Instrucciones

- GIVEN el detalle en la pestaña Ingredientes
- WHEN el usuario selecciona la pestaña Instrucciones
- THEN vuelve a mostrarse el texto de las instrucciones

#### Scenario: Receta sin instrucciones

- GIVEN una receta sin instrucciones
- WHEN el usuario abre su detalle
- THEN el panel de Instrucciones muestra un estado vacío claro
- AND la pestaña de Instrucciones sigue activa

#### Scenario: Edición de ingredientes dentro de su pestaña

- GIVEN el detalle en la pestaña Ingredientes
- WHEN el usuario pulsa «Corregir ingredientes», edita y guarda
- THEN la edición se realiza como hoy y los cambios se reflejan

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