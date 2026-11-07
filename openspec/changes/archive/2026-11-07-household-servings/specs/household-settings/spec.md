# Capability: household-settings

## ADDED Requirements

### Requirement: Preferencia de comensales persistida

El sistema SHALL persistir una preferencia de comensales habituales en la tabla
singleton `household_settings` (`id = 1`), con `default_servings` entre 1 y 20 y
valor por defecto 2. `GET /api/settings` SHALL requerir `read:recipes` y
devolver `{ "default_servings": n }`. `PUT /api/settings` SHALL requerir
`write:recipes`, validar el rango y responder `422` fuera de él.

#### Scenario: Lectura sin preferencia guardada

- GIVEN una base de datos recién migrada
- WHEN se consulta `GET /api/settings`
- THEN se devuelve `200` con `default_servings: 2`

#### Scenario: Actualización válida

- GIVEN `default_servings: 2`
- WHEN se envía `PUT /api/settings` con `default_servings: 4`
- THEN se devuelve `200` con el valor actualizado
- AND una lectura posterior devuelve `4`

#### Scenario: Valor fuera de rango

- GIVEN cualquier preferencia guardada
- WHEN se envía `PUT /api/settings` con `0` o `21`
- THEN se devuelve `422` con el mensaje de validación
- AND la preferencia guardada no cambia

#### Scenario: Permisos

- GIVEN un token sin `read:recipes`
- WHEN se consulta `GET /api/settings`
- THEN se devuelve `403`
- GIVEN un token sin `write:recipes`
- WHEN se envía `PUT /api/settings`
- THEN se devuelve `403`

### Requirement: Comensales habituales en ajustes

La página `/settings` SHALL ofrecer un campo numérico «Comensales habituales»
(1..20) con el valor actual, guardado explícito y error visible si falla. SHALL
mostrar 2 cuando el servidor no devuelva la preferencia. La consulta SHALL
persistirse en IndexedDB para que el último valor conocido esté disponible sin
red.

#### Scenario: Guardar la preferencia

- GIVEN la página de ajustes con `default_servings: 2`
- WHEN el usuario introduce `4` y guarda
- THEN se invoca `PUT /api/settings` y se muestra confirmación

#### Scenario: Error al guardar

- GIVEN el servidor devuelve `422`
- WHEN el usuario guarda un valor inválido
- THEN se muestra el error y no se actualiza el valor mostrado

#### Scenario: Disponibilidad sin red

- GIVEN una preferencia leída con anterioridad
- WHEN la aplicación arranca sin red
- THEN el detalle de receta usa el último valor conocido
