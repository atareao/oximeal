## MODIFIED Requirements

### Requirement: Alta de receta

El sistema SHALL exponer `POST /api/recipes` que valida categoría (main|salad|side|dessert|breakfast) e ingredientes, persiste la receta y SHALL aceptar un campo opcional `tags` con los nombres de sus etiquetas manuales, creando al vuelo las que no existan. SHALL aplicar el etiquetado automático salvo que `auto_tags` sea `false` y SHALL aceptar `suppressed_auto_tags`. La `Recipe` devuelta SHALL incluir sus etiquetas con `source`.

#### Scenario: Creación válida

- GIVEN un payload de receta válido
- WHEN se envía a `POST /api/recipes`
- THEN se crea la receta con id y se devuelve 201

#### Scenario: Categoría inválida

- GIVEN un payload con categoría no permitida
- WHEN se envía
- THEN se devuelve 400 y no se persiste nada

#### Scenario: Alta con etiquetas

- GIVEN un payload válido con `tags: ["Vegano"]` y la etiqueta no existía
- WHEN se envía a `POST /api/recipes`
- THEN se crea la etiqueta y la receta queda asociada a ella como `manual`
- AND la `Recipe` devuelta incluye la etiqueta y las automáticas resultantes

#### Scenario: Alta sin etiquetado automático

- GIVEN un payload válido con `auto_tags: false`
- WHEN se envía a `POST /api/recipes`
- THEN la receta no recibe etiquetas automáticas
- AND solo se asocian las etiquetas manuales indicadas

### Requirement: Detalle de receta

El sistema SHALL exponer `GET /api/recipes/{id}` devolviendo la receta completa con ingredientes y etiquetas, cada etiqueta con `id`, `name`, `slug` y `source`.

#### Scenario: No encontrada

- GIVEN un id inexistente
- WHEN se consulta el detalle
- THEN se devuelve 404

#### Scenario: Detalle con etiquetas

- GIVEN una receta con etiquetas manuales y automáticas
- WHEN se consulta `GET /api/recipes/{id}`
- THEN la respuesta incluye las etiquetas con `id`, `name`, `slug` y `source`

### Requirement: Actualización de receta

El sistema SHALL exponer `PUT /api/recipes/{id}` con permiso `write:recipes`, que reemplaza la receta completa —nombre, categoría, `diet_type`, `prep_time_minutes`, `servings`, ingredientes, instrucciones, `source_url` y etiquetas manuales—, responde `200` con la `Recipe` actualizada y mantiene sincronizado el índice de búsqueda FTS. El campo `tags` SHALL reemplazar el conjunto de etiquetas `manual`, y el etiquetado automático SHALL recalcularse salvo que `auto_tags` sea `false`.

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

#### Scenario: Reemplazo de etiquetas

- GIVEN una receta asociada manualmente a las etiquetas A y B
- WHEN se envía `PUT /api/recipes/{id}` con `tags: ["B", "C"]`
- THEN la receta queda asociada a B y C como `manual`
- AND la asociación manual con A desaparece
- AND no se borra ninguna etiqueta del catálogo
