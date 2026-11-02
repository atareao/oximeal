# Capability: recipes

## ADDED Requirements

### Requirement: Actualización de receta

El sistema SHALL exponer `PUT /api/recipes/{id}` con permiso `write:recipes`, que
reemplaza la receta completa —nombre, categoría, `diet_type`,
`prep_time_minutes`, `servings`, ingredientes, instrucciones y `source_url`—,
responde `200` con la `Recipe` actualizada y mantiene sincronizado el índice de
búsqueda FTS.

#### Scenario: Actualización válida

- GIVEN una receta existente y un payload válido con todos los campos
- WHEN se envía a `PUT /api/recipes/{id}`
- THEN se reemplaza la receta y se devuelve 200 con la `Recipe` actualizada
- AND el índice FTS refleja el cambio y el listado devuelve los datos nuevos

#### Scenario: Receta inexistente

- GIVEN un id que no existe
- WHEN se envía un payload válido a `PUT /api/recipes/{id}`
- THEN se devuelve 404
- AND no se crea ni modifica ninguna receta

#### Scenario: Payload inválido

- GIVEN una receta existente y un payload con categoría no permitida o campos inválidos
- WHEN se envía a `PUT /api/recipes/{id}`
- THEN se devuelve 400
- AND la receta original permanece sin cambios

#### Scenario: Actualización sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `PUT /api/recipes/{id}` sin el permiso `write:recipes`
- THEN se devuelve 401 si no hay sesión o 403 si falta el permiso
- AND no se modifica ninguna receta

### Requirement: Borrado de receta

El sistema SHALL exponer `DELETE /api/recipes/{id}` con permiso `write:recipes`,
que borra la receta y sus ingredientes de forma atómica, responde `204` y elimina
la receta del listado `GET /api/recipes`.

#### Scenario: Borrado válido

- GIVEN una receta existente con ingredientes
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se borran la receta y sus ingredientes y se devuelve 204
- AND la receta deja de aparecer en `GET /api/recipes`

#### Scenario: Receta inexistente

- GIVEN un id que no existe
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se devuelve 404
- AND no se borra ninguna otra receta

#### Scenario: Receta referenciada por el plan

- GIVEN una receta usada por una entrada del menú semanal
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se devuelve 409 Conflict
- AND la receta y la entrada del plan permanecen sin cambios

#### Scenario: Borrado sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `DELETE /api/recipes/{id}` sin el permiso `write:recipes`
- THEN se devuelve 401 si no hay sesión o 403 si falta el permiso
- AND la receta permanece sin cambios
