# meal-plan Specification

## Purpose
Esta capacidad gestiona la planificación semanal de comidas. Permite consultar el plan por rango de fechas, asignar recetas a franjas horarias (slots) y eliminar asignaciones existentes.

## Requirements

### Requirement: Consulta de plan por rango
El sistema SHALL exponer `GET /api/meal-plan?start_date=&end_date=` devolviendo las asignaciones del rango con su receta.

#### Scenario: Rango válido
- GIVEN asignaciones en un rango de fechas
- WHEN se consulta el rango
- THEN se devuelven las entradas ordenadas por fecha y tipo de comida

### Requirement: Asignación de slot

El sistema SHALL exponer `POST /api/meal-plan` (permiso `write:plan`) validando
`meal_type` (`lunch|dinner|breakfast|snack`), `role` (`main|salad|side`), una
`date` ISO válida, un `recipe_id` no vacío **y existente**, y `servings >= 1`
cuando se indique (por defecto `2`). Responde `201` con la `MealPlanEntryView`
creada y `400` si algún campo es inválido o la receta no existe, sin persistir
nada en ese caso.

#### Scenario: Receta inexistente
- GIVEN un `recipe_id` que no existe
- WHEN se envía la asignación a `POST /api/meal-plan`
- THEN se devuelve error de validación y no se persiste la entrada

#### Scenario: Asignación válida
- GIVEN una receta existente y un cuerpo con `meal_type`, `role`, `date` y `recipe_id` válidos
- WHEN se envía a `POST /api/meal-plan`
- THEN se crea la entrada y se devuelve `201` con la `MealPlanEntryView`
- AND la entrada aparece en el rango consultado por `GET /api/meal-plan`

#### Scenario: Raciones por defecto
- GIVEN un cuerpo de asignación sin el campo `servings`
- WHEN se envía a `POST /api/meal-plan`
- THEN la entrada se crea con `servings` igual a `2`

#### Scenario: Raciones inválidas
- GIVEN un cuerpo de asignación con `servings` menor que `1`
- WHEN se envía a `POST /api/meal-plan`
- THEN se devuelve `400` y no se persiste la entrada

#### Scenario: Asignación sin permiso
- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `POST /api/meal-plan` sin el permiso `write:plan`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
- AND no se crea ninguna entrada

### Requirement: Eliminación de slot
El sistema SHALL exponer `DELETE /api/meal-plan/{id}`.

#### Scenario: Eliminación correcta
- GIVEN una entrada existente
- WHEN se elimina por id
- THEN se devuelve 204 y deja de aparecer en el rango

### Requirement: Edición de raciones

El sistema SHALL exponer `PATCH /api/meal-plan/{id}` con permiso `write:plan`,
que actualiza las **raciones** de una entrada existente. El cuerpo SHALL incluir
`servings`; el handler responde `200` con la `MealPlanEntryView` actualizada,
`400` si `servings < 1` o el cuerpo es inválido, y `404` si la entrada no
existe. El reemplazo de `recipe_id`, `date`, `meal_type` o `role` queda fuera de
este contrato y se resuelve quitando y volviendo a añadir la entrada.

#### Scenario: Edición de raciones válida
- GIVEN una entrada existente con `servings` igual a `2`
- WHEN se envía a `PATCH /api/meal-plan/{id}` un cuerpo con `servings` igual a `4`
- THEN se devuelve `200` con la `MealPlanEntryView` y `servings` igual a `4`
- AND la entrada conserva su `id`, su receta y su slot

#### Scenario: Raciones inválidas
- GIVEN una entrada existente
- WHEN se envía a `PATCH /api/meal-plan/{id}` un cuerpo con `servings` menor que `1`
- THEN se devuelve `400`
- AND las raciones de la entrada permanecen sin cambios

#### Scenario: Entrada inexistente
- GIVEN un id que no corresponde a ninguna entrada del plan
- WHEN se envía a `PATCH /api/meal-plan/{id}` con un `servings` válido
- THEN se devuelve `404`
- AND no se crea ninguna entrada

#### Scenario: Edición sin permiso
- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `PATCH /api/meal-plan/{id}` sin el permiso `write:plan`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
- AND las raciones de la entrada permanecen sin cambios

### Requirement: Eliminación con permiso

El sistema SHALL exponer `DELETE /api/meal-plan/{id}` con permiso `write:plan`,
que elimina la entrada y responde `204`; devuelve `404` si la entrada no existe.

#### Scenario: Eliminación sin permiso
- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `DELETE /api/meal-plan/{id}` sin el permiso `write:plan`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
- AND la entrada permanece en el plan
