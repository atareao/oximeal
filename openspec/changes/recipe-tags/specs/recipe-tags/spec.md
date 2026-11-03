# Capability: recipe-tags

## ADDED Requirements

### Requirement: Modelo de etiquetas libres

El sistema SHALL modelar las etiquetas como **etiquetas libres** con relación
many-to-many entre recetas y etiquetas: una tabla `tags` con identificador,
`name` de presentación y una clave canónica `slug` con restricción `UNIQUE`, y
una tabla de unión `recipe_tags(recipe_id, tag_id)` con clave primaria compuesta
y sendas claves foráneas `ON DELETE CASCADE` a `recipes(id)` y `tags(id)`. El
`slug` SHALL ser el nombre normalizado (recorte de espacios extremos, colapso de
espacios internos y minúsculas Unicode; los diacríticos se conservan), de modo
que la unicidad sea **insensible a mayúsculas y a espacios**. Las etiquetas SHALL
quedar **fuera** del índice de búsqueda de texto completo `recipes_fts`.

#### Scenario: Migración 0007 crea el esquema de etiquetas

- GIVEN una base migrada hasta `0006`
- WHEN se aplica la migración `0007`
- THEN existen las tablas `tags` y `recipe_tags` y el índice `idx_recipe_tags_tag_id` sobre `recipe_tags(tag_id)`
- AND `tags.slug` tiene una restricción `UNIQUE`

#### Scenario: Unicidad insensible a mayúsculas y espacios

- GIVEN una etiqueta existente cuyo `slug` es `sin-gluten`
- WHEN se intenta crear una etiqueta con el nombre "Sin Gluten" o "sin  gluten"
- THEN no se crea una segunda etiqueta
- AND se devuelve la etiqueta existente

#### Scenario: Borrado en cascada de asociaciones

- GIVEN una receta con etiquetas asociadas
- WHEN se borra la receta
- THEN sus filas en `recipe_tags` desaparecen
- AND las etiquetas siguen existiendo en `tags`

#### Scenario: Las etiquetas no entran en el índice FTS

- GIVEN una receta etiquetada con "vegano" cuyo nombre e instrucciones no contienen esa palabra
- WHEN se consulta `GET /api/recipes?q=vegano`
- THEN la receta no aparece por coincidencia de texto
- AND el filtro por etiqueta es el mecanismo para encontrarla

#### Scenario: Migración aditiva y reversible

- GIVEN una base con recetas y datos existentes
- WHEN se aplica la migración `0007`
- THEN no se modifica ninguna fila de `recipes`
- AND revertirla (DROP de `recipe_tags` y `tags`) no pierde datos de recetas

### Requirement: Gestión de etiquetas

El sistema SHALL exponer la gestión de etiquetas. `GET /api/tags` (permiso
`read:recipes`) SHALL devolver la lista de etiquetas con su recuento de recetas.
`POST /api/tags` (permiso `write:recipes`) SHALL crear una etiqueta a partir de
`{ "name": "..." }` o, si ya existe una con el mismo `slug`, SHALL devolver la
existente de forma idempotente. `DELETE /api/tags/{id}` (permiso
`write:recipes`) SHALL borrar la etiqueta y sus asociaciones en cascada sin
borrar las recetas afectadas.

#### Scenario: Listado con conteo

- GIVEN etiquetas en uso y etiquetas sin uso
- WHEN se consulta `GET /api/tags`
- THEN se devuelven todas con `id`, `name`, `slug` y `recipe_count`
- AND las etiquetas sin recetas tienen `recipe_count` igual a `0`

#### Scenario: Creación de etiqueta

- GIVEN un nombre de etiqueta válido que aún no existe
- WHEN se envía `POST /api/tags`
- THEN se crea la etiqueta y se responde `201` con `id`, `name`, `slug` y `recipe_count` igual a `0`

#### Scenario: Creación duplicada idempotente

- GIVEN una etiqueta ya existente
- WHEN se envía `POST /api/tags` con un nombre que normaliza al mismo `slug`
- THEN no se crea una segunda etiqueta
- AND se responde `200` con la etiqueta existente

#### Scenario: Nombre vacío

- GIVEN un `name` vacío o compuesto solo de espacios
- WHEN se envía `POST /api/tags`
- THEN se responde `400` sin crear ninguna etiqueta

#### Scenario: Borrado de etiqueta con recetas asociadas

- GIVEN una etiqueta asociada a una o más recetas
- WHEN se envía `DELETE /api/tags/{id}`
- THEN se responde `204`
- AND la etiqueta y sus asociaciones desaparecen
- AND las recetas afectadas permanecen intactas

#### Scenario: Borrado de etiqueta inexistente

- GIVEN un id que no existe
- WHEN se envía `DELETE /api/tags/{id}`
- THEN se responde `404`

#### Scenario: Gestión sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía `POST /api/tags` o `DELETE /api/tags/{id}` sin el permiso `write:recipes`
- THEN se responde `401` si no hay sesión o `403` si falta el permiso
- AND no se modifica ninguna etiqueta

### Requirement: Etiquetas en el alta y la edición de recetas

El contrato de `POST /api/recipes` y `PUT /api/recipes/{id}` SHALL aceptar un
campo opcional `tags` con los nombres de las etiquetas. El sistema SHALL crear al
vuelo las etiquetas que no existan (comparando por `slug`) y SHALL **reemplazar**
el conjunto de etiquetas de la receta en cada alta o actualización. Las
respuestas `Recipe` y `RecipeSummary` SHALL incluir las etiquetas de la receta con
`id`, `name` y `slug`.

#### Scenario: Alta con etiquetas nuevas

- GIVEN un payload de receta con `tags: ["Vegano", "Sin gluten"]`
- WHEN se envía `POST /api/recipes`
- THEN se crean las etiquetas que no existían
- AND la `Recipe` devuelta incluye ambas etiquetas
- AND la receta queda asociada a ellas

#### Scenario: Edición que reemplaza etiquetas

- GIVEN una receta asociada a las etiquetas A y B
- WHEN se envía `PUT /api/recipes/{id}` con `tags: ["B", "C"]`
- THEN la receta queda asociada a B y C
- AND la asociación con A desaparece

#### Scenario: Quitar todas las etiquetas

- GIVEN una receta etiquetada
- WHEN se envía `PUT /api/recipes/{id}` con `tags: []`
- THEN la receta queda sin etiquetas
- AND las etiquetas siguen existiendo en el catálogo

#### Scenario: Nombres duplicados normalizados

- GIVEN un payload con `tags: ["Vegano", "vegano"]`
- WHEN se envía a `POST /api/recipes`
- THEN la receta queda asociada a una sola etiqueta

#### Scenario: Etiquetas en el listado y el detalle

- GIVEN recetas con etiquetas
- WHEN se consulta `GET /api/recipes` o `GET /api/recipes/{id}`
- THEN cada receta incluye sus etiquetas con `id`, `name` y `slug`

### Requirement: Filtro de recetas por etiqueta

`GET /api/recipes` SHALL aceptar un parámetro opcional `tag` que SHALL ser el `id`
o el `slug` de una etiqueta y SHALL devolver únicamente las recetas asociadas a
esa etiqueta. El filtro SHALL combinarse con `q`, `category` y la paginación, y el
`total` de la respuesta SHALL reflejar el conjunto filtrado.

#### Scenario: Filtro por slug

- GIVEN recetas etiquetadas con `sin-gluten` y recetas sin esa etiqueta
- WHEN se consulta `GET /api/recipes?tag=sin-gluten`
- THEN solo se devuelven las recetas con esa etiqueta
- AND el `total` cuenta únicamente esas recetas

#### Scenario: Filtro por id

- GIVEN una etiqueta con un id conocido
- WHEN se consulta `GET /api/recipes?tag=<id>`
- THEN se devuelven las recetas asociadas a esa etiqueta

#### Scenario: Filtro combinado con búsqueda y paginación

- GIVEN recetas etiquetadas que además coinciden con un texto de búsqueda
- WHEN se consulta `GET /api/recipes?tag=<slug>&q=<texto>&page=1&page_size=10`
- THEN se devuelven solo las recetas que cumplen ambos criterios, paginadas

#### Scenario: Etiqueta inexistente

- GIVEN un `tag` que no corresponde a ninguna etiqueta
- WHEN se consulta `GET /api/recipes?tag=<desconocido>`
- THEN se responde `200` con una lista vacía
