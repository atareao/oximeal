# Change: etiquetado automático de recetas (heurística, procedencia y UI)

## Why

El recetario ya soporta etiquetas libres manuales y su filtrado (capacidad `recipe-tags`), pero marcar cada receta a mano es tedioso y se olvida. Falta que el sistema sugiera automáticamente etiquetas de dieta e intolerancias a partir del contenido de la receta, conservando el usuario el control final (precedencia manual y supresión de sugerencias).

## What Changes

- Vocabulario cerrado de etiquetas automáticas: `vegetariano`, `vegano`, `sin-gluten`, `sin-lactosa` y `picante`, con `slug` estable y presentes siempre en el catálogo.
- Heurística determinista sobre los ingredientes (reforzada por menciones en `name`/`instructions`), con normalización de texto (minúsculas sin diacríticos) y coincidencia por token completo.
- Procedencia por asociación: nueva columna `recipe_tags.source` (`manual`|`auto`); la manual tiene precedencia sobre la automática.
- Supresión por receta: tabla `recipe_tag_suppressions` para retirar una sugerencia sin que se reasigne.
- API: `RecipeTag` expone `source`; `RecipeInput` acepta `auto_tags` (booleano, por defecto `true`) y `suppressed_auto_tags`; nuevo `POST /api/recipes/{id}/autotag`.
- Migración `0009_recipe_tag_provenance.sql` (aditiva) más backfill idempotente en arranque de las recetas existentes.
- UI: el formulario muestra y permite retirar sugerencias; la tarjeta y el detalle distinguen visualmente etiqueta manual de automática.

## Capabilities

### New Capabilities

- _(ninguna; el etiquetado automático se modela dentro de la capacidad existente `recipe-tags`)_

### Modified Capabilities

- `recipe-tags`: catálogo automático, heurística, procedencia y precedencia, supresión, backfill y endpoint de recálculo; ajuste del alta y edición de etiquetas.
- `recipes`: `RecipeTag` con `source`; `RecipeInput` con `auto_tags` y `suppressed_auto_tags`; `Recipe` y `RecipeSummary` con etiquetas etiquetadas por procedencia.
- `recipe-catalog`: sugerencias automáticas en el formulario, supresión y distinción visual manual/automática en tarjeta y detalle.

## Impact

- Backend: `backend/migrations/0009_recipe_tag_provenance.sql` (nueva); `backend/src/services/autotag.rs` (nueva); `backend/src/models/tag.rs`; `backend/src/models/recipe.rs`; `backend/src/repo/tags.rs`; `backend/src/repo/recipes.rs`; `backend/src/routes/tags.rs`; `backend/src/routes/recipes.rs`; `backend/src/routes/openapi.rs`; punto de arranque para el backfill; tests de integración en `backend/tests/`.
- Frontend: `frontend/src/api/types.ts`, `frontend/src/api/client.ts`, `frontend/src/api/hooks.ts`, `frontend/src/components/RecipeForm.tsx`, `frontend/src/components/RecipeCard.tsx`, `frontend/src/pages/RecipeDetailPage.tsx` y sus tests.
- Sin dependencias nuevas. Sin cambios en el esquema FTS.
- Continúa el issue #55 (backlog UI #3).
