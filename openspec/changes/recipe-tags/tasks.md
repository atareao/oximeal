# Tasks — Etiquetas libres de recetas (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [ ] 1.1 Leer `backend/migrations/0001_init.sql`…`0006_shopping_item_client_id.sql` y confirmar el estilo (FKs `ON DELETE` explícitas, `CHECK`, índices, WAL, `foreign_keys=on`)
- [ ] 1.2 Leer `backend/src/models/recipe.rs`, `backend/src/repo/recipes.rs`, `backend/src/routes/recipes.rs`, `backend/src/routes/mod.rs`, `backend/src/routes/validation.rs` y `backend/src/routes/openapi.rs`
- [ ] 1.3 Leer `openspec/specs/recipe-search/spec.md` (triggers `ai/ad/au`, tokenizer trigram) y confirmar que las etiquetas quedan fuera del índice FTS
- [ ] 1.4 Leer `frontend/src/api/types.ts`, `client.ts`, `hooks.ts`, `frontend/src/pages/RecipesPage.tsx`, `RecipeDetailPage.tsx`, `frontend/src/components/RecipeForm.tsx` y `RecipeCard.tsx`
- [ ] 1.5 Confirmar el patrón de degradado offline (`offline-data`/`offline-writes`) y el uso de query params en el recetario

## 2. RED

- [ ] 2.1 Test integración backend: la migración `0007` crea `tags`, `recipe_tags`, `idx_recipe_tags_tag_id` y la unicidad de `tags(slug)`
- [ ] 2.2 Test integración backend: `GET /api/tags` devuelve etiquetas con `recipe_count`
- [ ] 2.3 Test integración backend: `POST /api/tags` crea (`201`); nombre duplicado normalizado devuelve `200` con la existente; nombre vacío `400`
- [ ] 2.4 Test integración backend: `DELETE /api/tags/{id}` con recetas asociadas responde `204`, borra asociaciones y conserva recetas; id inexistente `404`
- [ ] 2.5 Test integración backend: `POST /api/tags` y `DELETE /api/tags/{id}` sin permiso devuelven `401`/`403`
- [ ] 2.6 Test integración backend: `POST`/`PUT /api/recipes` con `tags` crean al vuelo, asocian y reemplazan; `PUT` con `tags: []` las quita; duplicados normalizados deduplican
- [ ] 2.7 Test integración backend: `GET /api/recipes` y `GET /api/recipes/{id}` incluyen `tags` con `id`, `name` y `slug`
- [ ] 2.8 Test integración backend: `GET /api/recipes?tag=<slug|id>` filtra, combina con `q`/`category`/paginación y devuelve `200` vacío con tag inexistente
- [ ] 2.9 Test integración backend: las etiquetas no aparecen en la búsqueda FTS por texto
- [ ] 2.10 Test frontend (vitest + Testing Library): el selector de etiquetas del `RecipeForm` permite seleccionar existentes y crear al vuelo; envía los nombres
- [ ] 2.11 Test frontend: `RecipeCard` y el detalle muestran las etiquetas
- [ ] 2.12 Test frontend: `RecipesPage` filtra por etiqueta, refleja el filtro en la URL y lo combina con búsqueda y paginación
- [ ] 2.13 Test frontend: accesibilidad (objetivos ≥48px, etiquetas/roles) y degradado offline del filtro/selector
- [ ] 2.14 Ejecutar `cargo test` y `npx vitest run` y confirmar el estado RED

## 3. GREEN

- [ ] 3.1 Migración `backend/migrations/0007_recipe_tags.sql` (aditiva): `tags`, `recipe_tags`, FKs `ON DELETE CASCADE`, PK compuesta, `idx_recipe_tags_tag_id` y `slug UNIQUE`; sin tocar `recipes_fts`
- [ ] 3.2 Backend: `backend/src/models/tag.rs` (DTOs y filas) y `backend/src/repo/tags.rs` (listar con conteo, alta idempotente por `slug`, borrado en cascada)
- [ ] 3.3 Backend: `backend/src/routes/tags.rs` (`GET`/`POST`/`DELETE`) y registro en `read_router`/`write_router` con `read:recipes`/`write:recipes`
- [ ] 3.4 Backend: extender `RecipeInput`/`Recipe`/`RecipeSummary` con `tags` y persistir la unión en `repo::recipes` (alta/actualización con reemplazo, `BEGIN IMMEDIATE`)
- [ ] 3.5 Backend: parámetro `tag` en `RecipeQuery` y en `repo::recipes::list` (resolución por id o slug, combinado con búsqueda y paginación)
- [ ] 3.6 Backend: documentar los nuevos endpoints y esquemas en `backend/src/routes/openapi.rs`
- [ ] 3.7 Frontend: tipos/cliente/hooks (`fetchTags`, `createTag`, `deleteTag`, `useTags`, `useCreateTag`) e invalidación del catálogo
- [ ] 3.8 Frontend: selector de etiquetas en `RecipeForm`, etiquetas en `RecipeCard` y detalle, y filtro por etiqueta en `RecipesPage` reflejado en la URL
- [ ] 3.9 `cargo test` y `npx vitest run` en verde; `cargo check` y `npx tsc --noEmit` sin errores

## 4. REFACTOR

- [ ] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings`
- [ ] 4.2 `npm run lint` y `pnpm format:check` en el frontend
- [ ] 4.3 Re-ejecutar `cargo test` y `npx vitest run` para descartar regresiones

## 5. Cierre

- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` (ADR-35, Fase 7) y `CHANGELOG.md`
- [ ] 5.3 Archivar el change con `openspec archive recipe-tags`
