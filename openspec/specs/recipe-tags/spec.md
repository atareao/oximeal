# recipe-tags Specification

## Purpose

Esta capacidad modela las **etiquetas libres** de las recetas como relación
many-to-many, expone su gestión (`GET`/`POST`/`DELETE /api/tags`), permite
asignarlas por su nombre al crear o editar una receta y filtrar el recetario
por etiqueta, manteniéndolas fuera del índice de texto completo.

## Requirements

### Requirement: Modelo de etiquetas libres

El sistema SHALL modelar las etiquetas como **etiquetas libres** con relación
many-to-many entre recetas y etiquetas. El `slug` SHALL ser el nombre
normalizado (espacios recortados y colapsados, minúsculas Unicode, diacríticos
conservados), de modo que la unicidad sea **insensible a mayúsculas y a
espacios**. Las etiquetas SHALL quedar **fuera** del índice de búsqueda de texto
completo `recipes_fts`.

#### Scenario: Migración 0007 crea el esquema de etiquetas

- GIVEN una base migrada hasta `0006`
- WHEN se aplica la migración `0007`
- THEN existen las tablas `tags` (identificador, `name` y `slug`) y `recipe_tags(recipe_id, tag_id)` con clave primaria compuesta, y el índice `idx_recipe_tags_tag_id` sobre `recipe_tags(tag_id)`
- AND `tags.slug` tiene una restricción `UNIQUE`
- AND `recipe_tags` tiene claves foráneas `ON DELETE CASCADE` a `recipes(id)` y `tags(id)`

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

El contrato de `POST /api/recipes` y `PUT /api/recipes/{id}` SHALL aceptar un campo opcional `tags` con los nombres de las etiquetas manuales, creando al vuelo las que no existan por `slug`. Las etiquetas `manual` SHALL reemplazar el conjunto manual anterior y SHALL tener precedencia sobre la heurística automática, que SHALL añadir sus etiquetas `auto` en la misma operación salvo que `auto_tags` sea `false`. Las respuestas `Recipe` y `RecipeSummary` SHALL incluir las etiquetas con `id`, `name`, `slug` y `source`.

#### Scenario: Alta con etiquetas nuevas

- GIVEN un payload de receta con `tags: ["Vegano", "Sin gluten"]`
- WHEN se envía `POST /api/recipes`
- THEN se crean las etiquetas que no existían
- AND la `Recipe` devuelta incluye ambas etiquetas con `source` `manual`
- AND la receta queda asociada a ellas y a las automáticas resultantes

#### Scenario: Edición que reemplaza etiquetas

- GIVEN una receta asociada manualmente a las etiquetas A y B
- WHEN se envía `PUT /api/recipes/{id}` con `tags: ["B", "C"]`
- THEN la receta queda asociada a B y C como `manual`
- AND la asociación manual con A desaparece
- AND las etiquetas automáticas se recalculan sin tocar las manuales

#### Scenario: Quitar todas las etiquetas

- GIVEN una receta etiquetada manualmente
- WHEN se envía `PUT /api/recipes/{id}` con `tags: []`
- THEN la receta queda sin etiquetas manuales
- AND las etiquetas siguen existiendo en el catálogo

#### Scenario: Nombres duplicados normalizados

- GIVEN un payload con `tags: ["Vegano", "vegano"]`
- WHEN se envía a `POST /api/recipes`
- THEN la receta queda asociada a una sola etiqueta `manual`

#### Scenario: Etiquetas en el listado y el detalle

- GIVEN recetas con etiquetas manuales y automáticas
- WHEN se consulta `GET /api/recipes` o `GET /api/recipes/{id}`
- THEN cada receta incluye sus etiquetas con `id`, `name`, `slug` y `source`

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

### Requirement: Catálogo de etiquetas automáticas

El sistema SHALL mantener un vocabulario cerrado de etiquetas automáticas con `slug` estable —`vegetariano`, `vegano`, `sin-gluten`, `sin-lactosa` y `picante`— presentes en el catálogo `tags` de forma idempotente, aunque ninguna receta las use, y SHALL poder filtrarse como cualquier etiqueta libre.

#### Scenario: Catálogo creado por la migración

- GIVEN una base migrada hasta `0008`
- WHEN se aplica la migración `0009`
- THEN existen las cinco etiquetas con sus `slug` canónicos
- AND su creación es idempotente (no se duplican si ya existen)

#### Scenario: Filtrado por etiqueta automática

- GIVEN recetas etiquetadas automáticamente con `vegano`
- WHEN se consulta `GET /api/recipes?tag=vegano`
- THEN se devuelven las recetas asociadas

### Requirement: Heurística de etiquetado automático

El sistema SHALL calcular etiquetas automáticas desde los ingredientes, reforzados por menciones en `name` e `instructions`, normalizando el texto (minúsculas sin diacríticos) y comparando por token completo. Las etiquetas de exclusión (`vegetariano`, `vegano`, `sin-gluten`, `sin-lactosa`) SHALL asignarse cuando no se detecte ningún término vetado; `picante` SHALL asignarse cuando se detecte un término de su lista positiva. El cálculo SHALL ser determinista y no depender de red.

#### Scenario: Vegano por ausencia de origen animal

- GIVEN una receta cuyos ingredientes no incluyen carne, pescado, huevo, lácteos ni miel
- WHEN se etiqueta automáticamente
- THEN recibe `vegano` y `vegetariano`

#### Scenario: No vegetariano por presencia de carne o pescado

- GIVEN una receta con un ingrediente de carne o pescado
- WHEN se etiqueta automáticamente
- THEN no recibe `vegetariano` ni `vegano`

#### Scenario: Sin gluten por ausencia de cereales con gluten

- GIVEN una receta sin trigo, cebada, centeno, pan, pasta ni sémola
- WHEN se etiqueta automáticamente
- THEN recibe `sin-gluten`

#### Scenario: Picante por presencia

- GIVEN una receta con un ingrediente o mención picante
- WHEN se etiqueta automáticamente
- THEN recibe `picante`

#### Scenario: Coincidencia por token y no por subcadena

- GIVEN una receta con el ingrediente «panqueque» y ningún término con gluten
- WHEN se etiqueta automáticamente
- THEN «panqueque» no cuenta como «pan» y la receta recibe `sin-gluten`

### Requirement: Procedencia y precedencia de etiquetas

Cada asociación receta-etiqueta SHALL registrar su procedencia, `manual` o `auto`. Las etiquetas enviadas por el cliente SHALL almacenarse como `manual` y SHALL tener precedencia sobre la heurística: una etiqueta presente en ambas formas SHALL guardarse una sola vez como `manual`. El recálculo automático SHALL afectar solo a las asociaciones `auto`. `Recipe` y `RecipeSummary` SHALL exponer `source` en cada etiqueta.

#### Scenario: Manual gana sobre automática

- GIVEN una receta que la heurística marcaría `vegano`
- WHEN se guarda con `tags: ["Vegano"]`
- THEN existe una única asociación `vegano` con `source` `manual`

#### Scenario: El recálculo respeta lo manual

- GIVEN una receta con etiquetas `manual` y `auto`
- WHEN se recalcula el etiquetado automático
- THEN las asociaciones `manual` permanecen intactas
- AND solo cambian las `auto`

#### Scenario: Procedencia expuesta

- GIVEN una receta con etiquetas manuales y automáticas
- WHEN se consulta su detalle o el listado
- THEN cada etiqueta incluye `source` con su procedencia

### Requirement: Supresión de etiquetas automáticas

El sistema SHALL permitir suprimir por receta una etiqueta automática. Una etiqueta suprimida SHALL NOT volver a asignarse automáticamente; añadirla manualmente SHALL eliminar su supresión. `RecipeInput` SHALL aceptar `suppressed_auto_tags` (nombres o slugs suprimidos) y `auto_tags` (booleano, por defecto `true`) para desactivar el etiquetado automático de esa receta.

#### Scenario: Suprimir y no reasignar

- GIVEN una receta con la sugerencia `sin-gluten` suprimida
- WHEN se recalcula el etiquetado automático
- THEN `sin-gluten` no se vuelve a asignar

#### Scenario: Añadir manualmente quita la supresión

- GIVEN una receta con `sin-gluten` suprimida
- WHEN se guarda con `tags: ["Sin gluten"]`
- THEN la receta queda asociada a `sin-gluten` como `manual`
- AND la supresión desaparece

#### Scenario: Etiquetado automático desactivado

- GIVEN una receta guardada con `auto_tags: false`
- WHEN se persiste
- THEN no se añade ninguna etiqueta automática
- AND se conservan las manuales y las supresiones

### Requirement: Backfill de recetas existentes

Al aplicar la migración `0009`, las asociaciones preexistentes SHALL quedar marcadas como `manual`. El sistema SHALL ejecutar una pasada de etiquetado automático idempotente sobre las recetas sin asociaciones `auto`, una sola vez mediante un marcador persistente, sin duplicar etiquetas manuales ni reasignar suprimidas.

#### Scenario: Migración marca lo existente como manual

- GIVEN una base con asociaciones creadas antes de `0009`
- WHEN se aplica la migración
- THEN todas quedan con `source` `manual`

#### Scenario: Backfill único e idempotente

- GIVEN recetas existentes sin etiquetas automáticas
- WHEN arranca el sistema la primera vez tras `0009`
- THEN se calculan sus etiquetas automáticas
- AND en arranques posteriores no se repite el proceso

### Requirement: Recálculo de etiquetas de una receta

El sistema SHALL exponer `POST /api/recipes/{id}/autotag`, con permiso `write:recipes`, que recalcula las etiquetas automáticas de una receta y devuelve la `Recipe` actualizada. Un id inexistente SHALL responder `404`; sin permiso, `401` si no hay sesión o `403` si falta el permiso.

#### Scenario: Recálculo correcto

- GIVEN una receta existente
- WHEN se envía `POST /api/recipes/{id}/autotag`
- THEN se devuelve `200` con la `Recipe` y sus etiquetas automáticas actualizadas

#### Scenario: Receta inexistente

- GIVEN un id que no existe
- WHEN se envía `POST /api/recipes/{id}/autotag`
- THEN se devuelve `404`

#### Scenario: Recálculo sin permiso

- GIVEN una petición sin el permiso `write:recipes` y el modo abierto desactivado
- WHEN se envía `POST /api/recipes/{id}/autotag`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
