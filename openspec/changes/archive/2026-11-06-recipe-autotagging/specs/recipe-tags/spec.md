## ADDED Requirements

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

## MODIFIED Requirements

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
