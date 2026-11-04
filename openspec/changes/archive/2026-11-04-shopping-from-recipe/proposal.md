# Change: volcado de ingredientes de una receta a la lista de la compra

## Why

Hoy la lista de la compra solo se puede poblar de dos formas: a mano
(`POST /api/shopping-list`) o reconstruyéndola desde el menú semanal
(`POST /api/shopping-list/consolidate`, que usa el motor de
`ingredient-consolidation`). **No existe** la forma más natural: añadir a la
lista los ingredientes de **una receta concreta** que el usuario está viendo en
su detalle `frontend/src/pages/RecipeDetailPage.tsx`. Para hacerlo hay que
teclear cada ingrediente a mano, perdiendo la canonicalización de unidades, la
normalización de nombres y la categorización por pasillo que ya ofrece el motor
determinista.

Además, las recetas ya soportan `servings` y la consolidación desde el plan ya
escala por raciones (`ingredient-consolidation`), pero ese escalado no está
disponible para una receta suelta. Este change cierra el issue **#56** (change
P3 de la Fase 7 de producto) y ejecuta el **ADR-36** (volcado de receta a la
lista con escalado).

## What Changes

- Backend: nuevo endpoint `POST /api/shopping-list/from-recipe` con permiso
  `write:shopping`. El recurso modificado es la **lista** (no la receta), así que
  se agrupa con el resto de escrituras de `/api/shopping-list*` y comparte la
  fila `POST/PATCH/DELETE /api/shopping-list* | write:shopping` de la tabla de
  permisos de `backend/src/routes/mod.rs`.
- Contrato: cuerpo `{ "recipe_id": "<uuid>", "servings": <entero >= 1>, opcional }`.
  `servings` omitido toma las raciones de la receta (`recipes.servings`, por
  defecto `2`). Responde `200` con la lista agrupada resultante
  (`[ShoppingListGroup]`, igual que `consolidate`), `400` si `servings` es
  inválido y `404` si la receta no existe.
- Semántica declarativa e idempotente: se registra la receta como fuente
  (`shopping_recipe_sources`) y se **reconstruyen** todas las líneas derivadas
  de recetas reconsolidando las recetas registradas con el motor de
  `ingredient-consolidation`. Repetir la petición no duplica; cambiar las
  raciones reescala. Las líneas derivadas del plan (`source_plan_id`) y los
  ítems manuales no se tocan.
- Distinción de origen: nueva columna `source_recipe_id` en
  `shopping_list_items` (nula para manuales, presente para las derivadas de una
  receta), análoga a `source_plan_id`.
- Escalado: factor `raciones_solicitadas / raciones_receta`, aplicado a todas
  las cantidades antes de consolidar (consistente con el escalado desde el plan);
  las unidades no aditivas (`al gusto`) conservan cantidad 0 y las familias
  incompatibles no se suman.
- Frontend: acción «Añadir a la lista» en `RecipeDetailPage` con selector de
  raciones (por defecto las de la receta), feedback y enlace a `/shopping`;
  hook `useAddRecipeToShoppingList` y `addRecipeToShoppingList` en el cliente.
- Offline: la operación es compuesta y depende del servidor; **no** se encola en
  IndexedDB y la UI la deshabilita sin conexión, coherente con `offline-writes`.

## Impact

- Capabilities:
  - `shopping-list` (ADDED: volcado de receta, idempotencia/deduplicación;
    MODIFIED: consolidación desde el plan conserva las líneas de recetas).
  - `ingredient-consolidation` (ADDED: escalado por raciones en el volcado).
  - `recipe-catalog` (ADDED: acción «Añadir a la lista» en el detalle).
  - `offline-writes` (ADDED: operaciones compuestas no encoladas).
- Backend: `backend/src/routes/shopping_list.rs`, `backend/src/routes/mod.rs`
  (doc de la tabla de permisos), `backend/src/models/shopping_list.rs`,
  `backend/src/repo/shopping_list.rs`, nueva migración para `source_recipe_id` y
  `shopping_recipe_sources`, y tests de integración en `backend/tests/`.
- Frontend: `frontend/src/pages/RecipeDetailPage.tsx`,
  `frontend/src/api/{client,hooks,types}.ts` y tests de componente/hook.
- Sin dependencias nuevas.
- Cierra el issue #56. ADR-36.
