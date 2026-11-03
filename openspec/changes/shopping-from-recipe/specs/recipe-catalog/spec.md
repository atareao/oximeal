# Capability: recipe-catalog

## ADDED Requirements

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
