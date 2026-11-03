# Capability: recipes

## MODIFIED Requirements

### Requirement: Listado y búsqueda de recetas

El sistema SHALL exponer `GET /api/recipes` con filtro de texto opcional `q`
(FTS5 trigram), filtro `category` y filtro `tag` (id o slug de etiqueta), con
paginación. El `total` de la respuesta SHALL reflejar el conjunto que cumple los
filtros aplicados.

#### Scenario: Búsqueda por substring

- GIVEN recetas que contienen "tomate" en nombre o instrucciones
- WHEN se consulta `GET /api/recipes?q=tomate`
- THEN se devuelven las recetas coincidentes con metadatos de paginación

#### Scenario: Query corta

- GIVEN una query de menos de 3 caracteres
- WHEN se consulta el listado
- THEN el sistema usa fallback LIKE en lugar de MATCH

#### Scenario: Filtro por etiqueta

- GIVEN recetas asociadas a la etiqueta `sin-gluten`
- WHEN se consulta `GET /api/recipes?tag=sin-gluten`
- THEN se devuelven solo las recetas asociadas a esa etiqueta
- AND el `total` refleja ese subconjunto

#### Scenario: Filtros combinados

- GIVEN recetas que cumplen parte de una búsqueda de texto y de un filtro de categoría y etiqueta
- WHEN se consultan `q`, `category` y `tag` a la vez
- THEN se devuelven solo las recetas que cumplen todos los criterios, paginadas

### Requirement: Alta de receta

El sistema SHALL exponer `POST /api/recipes` que valida categoría
(main|salad|side|dessert|breakfast) e ingredientes, persiste la receta y SHALL
aceptar un campo opcional `tags` con los nombres de sus etiquetas, creando al
vuelo las que no existan y asociándolas a la receta. La `Recipe` devuelta SHALL
incluir sus etiquetas.

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
- THEN se crea la etiqueta y la receta queda asociada a ella
- AND la `Recipe` devuelta incluye la etiqueta

### Requirement: Detalle de receta

El sistema SHALL exponer `GET /api/recipes/{id}` devolviendo la receta completa
con ingredientes y etiquetas.

#### Scenario: No encontrada

- GIVEN un id inexistente
- WHEN se consulta el detalle
- THEN se devuelve 404

#### Scenario: Detalle con etiquetas

- GIVEN una receta con etiquetas asociadas
- WHEN se consulta `GET /api/recipes/{id}`
- THEN la respuesta incluye las etiquetas con `id`, `name` y `slug`

### Requirement: Actualización de receta

El sistema SHALL exponer `PUT /api/recipes/{id}` con permiso `write:recipes`, que
reemplaza la receta completa —nombre, categoría, `diet_type`,
`prep_time_minutes`, `servings`, ingredientes, instrucciones, `source_url` y
etiquetas—, responde `200` con la `Recipe` actualizada y mantiene sincronizado el
índice de búsqueda FTS. El campo `tags` SHALL reemplazar el conjunto de etiquetas
de la receta, creando al vuelo las que no existan.

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

- GIVEN una receta asociada a las etiquetas A y B
- WHEN se envía `PUT /api/recipes/{id}` con `tags: ["B", "C"]`
- THEN la receta queda asociada a B y C
- AND la asociación con A desaparece
- AND no se borra ninguna etiqueta del catálogo
