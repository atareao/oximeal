# Tasks — Etiquetado automático de recetas (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Leer `backend/migrations/0007_recipe_tags.sql` y `0008_shopping_recipe_sources.sql` para confirmar estilo y numeración (la siguiente es `0009`)
- [x] 1.2 Leer `backend/src/models/tag.rs`, `backend/src/models/recipe.rs`, `backend/src/repo/tags.rs`, `backend/src/repo/recipes.rs`, `backend/src/routes/recipes.rs`, `backend/src/routes/tags.rs` y `backend/src/routes/openapi.rs`
- [x] 1.3 Localizar el punto de arranque, la ejecución de migraciones y el marcador persistente disponible para el backfill
- [x] 1.4 Leer `frontend/src/components/RecipeForm.tsx`, `frontend/src/components/RecipeCard.tsx`, `frontend/src/pages/RecipeDetailPage.tsx`, `frontend/src/api/types.ts`, `frontend/src/api/client.ts` y `frontend/src/api/hooks.ts`

## 2. RED

- [x] 2.1 Test de migración `0009`: columna `source` con `DEFAULT 'manual'`, tabla `recipe_tag_suppressions` y catálogo de las cinco etiquetas automáticas
- [x] 2.2 Test unitario de la heurística (`services::autotag`): exclusión por ausencia (vegano, vegetariano, sin-gluten, sin-lactosa), presencia (picante) y coincidencia por token
- [x] 2.3 Test de integración: el alta asigna etiquetas automáticas; la manual gana; `auto_tags: false` las desactiva
- [x] 2.4 Test de integración: la supresión impide reasignar; añadir la etiqueta manualmente quita la supresión
- [x] 2.5 Test de integración: `Recipe` y `RecipeSummary` exponen `source` en cada etiqueta
- [x] 2.6 Test de integración: `POST /api/recipes/{id}/autotag` recalcula, devuelve `404` y `401`/`403` según corresponda
- [x] 2.7 Test de integración: backfill único e idempotente
- [x] 2.8 Test de frontend: el formulario muestra sugerencias, permite retirarlas (`suppressed_auto_tags`) y desactivar (`auto_tags`)
- [x] 2.9 Test de frontend: tarjeta y detalle distinguen etiqueta manual de automática
- [x] 2.10 Ejecutar `cargo test` y `npx vitest run` y confirmar el estado RED

## 3. GREEN

- [x] 3.1 Migración `backend/migrations/0009_recipe_tag_provenance.sql`
- [x] 3.2 `backend/src/services/autotag.rs` (reglas, `apply` y `backfill`) y registro del módulo
- [x] 3.3 `backend/src/models/tag.rs` (`RecipeTag.source`) y `backend/src/models/recipe.rs` (`auto_tags`, `suppressed_auto_tags`)
- [x] 3.4 `backend/src/repo/tags.rs`: procedencia, supresión y catálogo automático
- [x] 3.5 `backend/src/repo/recipes.rs`: integrar `autotag::apply` en alta y edición dentro de la transacción
- [x] 3.6 `backend/src/routes/recipes.rs`: `POST /api/recipes/{id}/autotag` y documentación en OpenAPI
- [x] 3.7 Arranque: ejecutar `backfill` con marcador persistente
- [x] 3.8 Frontend: tipos, cliente y hooks, y UI (formulario, tarjeta y detalle)
- [x] 3.9 `cargo test`, `npx vitest run`, `cargo check` y `npx tsc --noEmit` en verde

## 4. REFACTOR

- [x] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings`
- [x] 4.2 `npm run lint` y formato del frontend
- [x] 4.3 Re-ejecutar `cargo test` y `npx vitest run` para descartar regresiones

## 5. Cierre

- [x] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [x] 5.2 Actualizar `PLAN.md` y `CHANGELOG.md`
- [x] 5.3 Archivar el change con `openspec archive recipe-autotagging`
