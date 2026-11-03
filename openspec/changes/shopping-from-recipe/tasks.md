# Tasks — Volcado de receta a la lista (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Leer `openspec/specs/{shopping-list,ingredient-consolidation,recipe-catalog,offline-writes}/spec.md` y los deltas de este change
- [ ] 1.2 Leer `backend/src/routes/shopping_list.rs`, `backend/src/routes/mod.rs`, `backend/src/models/shopping_list.rs` y `backend/src/repo/shopping_list.rs`
- [ ] 1.3 Leer `backend/src/domain/{consolidate,ingredient,units,normalize}.rs` y la migración de `shopping_list_items` (`backend/migrations/`)
- [ ] 1.4 Leer `frontend/src/pages/RecipeDetailPage.tsx`, `frontend/src/api/{client,hooks,types}.ts` y `frontend/src/offline/mutationQueue.ts`
- [ ] 1.5 Confirmar la forma del contrato: `POST /api/shopping-list/from-recipe` con `{ recipe_id, servings? }`, permiso `write:shopping`, `200` con `[ShoppingListGroup]`, `400`/`404`

## 2. RED
- [ ] 2.1 Test integración backend: `POST /api/shopping-list/from-recipe` feliz devuelve `200` con las líneas escaladas y `source_recipe_id`
- [ ] 2.2 Test integración backend: sin `servings` usa las raciones de la receta (factor 1)
- [ ] 2.3 Test integración backend: `servings` inválido (0/negativo/no entero) devuelve `400` y no modifica la lista
- [ ] 2.4 Test integración backend: `recipe_id` inexistente devuelve `404` y no modifica la lista
- [ ] 2.5 Test integración backend: repetición idéntica es idempotente (no duplica) y el cambio de raciones reescala
- [ ] 2.6 Test integración backend: dos recetas volcadas fusionan por nombre canónico + unidad
- [ ] 2.7 Test integración backend: no aditivos ("al gusto" vs "1 pizca") quedan en líneas separadas
- [ ] 2.8 Test integración backend: `POST /consolidate` conserva las líneas de receta y los ítems manuales
- [ ] 2.9 Test integración backend: el endpoint exige `write:shopping` (`401`/`403`)
- [ ] 2.10 Test frontend (vitest + Testing Library): «Añadir a la lista» envía `recipe_id` y `servings` y muestra confirmación con enlace a `/shopping`
- [ ] 2.11 Test frontend: el selector de raciones parte de las de la receta; error del servidor se muestra sin romper la vista
- [ ] 2.12 Test frontend: sin conexión la acción está deshabilitada y no se encola
- [ ] 2.13 Test frontend: accesibilidad (objetivo ≥48 px, etiquetas/roles) de la acción y el selector
- [ ] 2.14 Ejecutar `cargo test` y `npx vitest run` y confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Migración: columna `source_recipe_id` en `shopping_list_items` y tabla `shopping_recipe_sources` (con índices)
- [ ] 3.2 Modelos: `RecipeToShoppingListInput` en `backend/src/models/shopping_list.rs` (espejo en `frontend/src/api/types.ts`)
- [ ] 3.3 Repo: registrar la fuente (upsert) y reconstruir las líneas derivadas de recetas reconsolidando todas las fuentes (`BEGIN IMMEDIATE`), reutilizando `collect_drafts`/motor de dominio con escalado
- [ ] 3.4 Ruta `POST /api/shopping-list/from-recipe` en `backend/src/routes/shopping_list.rs` con validación de `servings` y `404` de receta; registrar en `write_router` y documentar la fila en `backend/src/routes/mod.rs`
- [ ] 3.5 Frontend: `addRecipeToShoppingList` en `client.ts` y `useAddRecipeToShoppingList` en `hooks.ts` (invalida `queryKeys.shoppingList.all`)
- [ ] 3.6 Frontend: acción «Añadir a la lista» + selector de raciones en `RecipeDetailPage.tsx`, con confirmación/enlace y deshabilitado offline
- [ ] 3.7 `cargo test` y `npx vitest run` en verde; `cargo check` y `npx tsc --noEmit` sin errores

## 4. REFACTOR
- [ ] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings`
- [ ] 4.2 `npm run lint` y `pnpm format:check` en el frontend
- [ ] 4.3 Re-ejecutar `cargo test` y `npx vitest run` para descartar regresiones

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate shopping-from-recipe --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` y `CHANGELOG.md` (issue #56, ADR-36)
- [ ] 5.3 Archivar el change con `openspec archive shopping-from-recipe`
