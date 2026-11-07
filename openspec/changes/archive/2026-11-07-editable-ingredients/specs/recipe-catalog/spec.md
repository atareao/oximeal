# Spec Delta

## ADDED Requirements

### Requirement: Edición de ingredientes en el detalle
El detalle de una receta SHALL permitir editar los ingredientes en la propia vista —nombre, cantidad y unidad— sin abrir el formulario completo, persistiendo con `PUT /api/recipes/{id}` y reflejando el resultado. SHALL mantener coherente el escalado de raciones tras la edición y degradar en modo offline según `offline-writes`.

#### Scenario: Corregir cantidad y unidad
- GIVEN el detalle con el ingrediente «queso parmesano» en `1 ud`
- WHEN el usuario cambia la cantidad a 40, la unidad a «g» y guarda
- THEN se envía `PUT /api/recipes/{id}` con el ingrediente corregido
- AND el detalle muestra «queso parmesano 40 g»

#### Scenario: Añadir y quitar ingredientes
- GIVEN el detalle de una receta
- WHEN el usuario añade un ingrediente o quita uno existente
- THEN el cambio se persiste con `PUT /api/recipes/{id}` y el detalle se actualiza

#### Scenario: Validación de la edición
- GIVEN una edición con el nombre vacío o la cantidad negativa
- WHEN el usuario intenta guardar
- THEN se muestra el error y no se envía ninguna petición

#### Scenario: Edición sin conexión
- GIVEN la aplicación sin conexión
- WHEN el usuario intenta editar un ingrediente
- THEN la edición se deshabilita o degrada sin prometer persistencia
