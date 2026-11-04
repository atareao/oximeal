# Change: etiquetas libres de recetas (modelo, API, filtro y UI)

## Why

El recetario de OxiMeal solo clasifica por `category` (main/salad/side/dessert/
breakfast) y `diet_type`: dos ejes cerrados que no cubren la forma real de
organizar un recetario doméstico (temporada, cocina, intolerancias, "para
invitados", etc.). El usuario no tiene ninguna vía para marcar y reencontrar
recetas por criterios propios, ni para filtrar el listado más allá del texto de
búsqueda y la categoría.

Este change introduce **etiquetas libres** (many-to-many), su gestión y su
integración en el recetario: asignarlas al crear/editar una receta, mostrarlas
en la tarjeta y el detalle, y **filtrar** el listado por etiqueta combinándolo
con la búsqueda FTS existente y la paginación. Cierra el issue #55 (Fase 7) y
materializa la decisión del ADR-35 (modelo de etiquetas).

## What Changes

- **Modelo de datos (migración `0007`):** se opta por etiquetas **libres** con
  relación many-to-many, en lugar de categorías cerradas, porque las categorías
  ya existen (`category`) y un vocabulario cerrado obligaría a decidir de
  antemano qué dimensiones importan. Se añaden:
  - `tags(id, name, slug, created_at)` con `slug UNIQUE` (clave canónica
    normalizada: recorte, colapso de espacios y minúsculas Unicode; los
    diacríticos se conservan). `name` guarda la etiqueta de presentación.
  - `recipe_tags(recipe_id, tag_id)` con clave primaria compuesta, FKs
    `ON DELETE CASCADE` a `recipes` y `tags`, e índice `idx_recipe_tags_tag_id`
    sobre `tag_id`.
  - Migración **aditiva** (no reconstruye tablas), reversible con DROP de las dos
    tablas nuevas; no toca `recipes` ni el índice FTS.
- **FTS:** las etiquetas **NO entran** en `recipes_fts` (que indexa
  `name`/`instructions` con tokenizer trigram). Son un filtro discreto por
  igualdad, no texto libre; indexarlas contaminaría las coincidencias por
  subcadena y exigiría retocar los triggers `ai/ad/au`. El filtro por etiqueta
  usa la tabla de unión y su índice.
- **API de etiquetas:** `GET /api/tags` (permiso `read:recipes`) con conteo de
  recetas; `POST /api/tags` (permiso `write:recipes`) que crea o devuelve la
  existente de forma idempotente; `DELETE /api/tags/{id}` (permiso
  `write:recipes`) que borra la etiqueta y sus asociaciones en cascada sin
  borrar recetas.
- **Tags en receta:** `POST`/`PUT /api/recipes` aceptan `tags` (nombres), crean
  al vuelo las que falten y **reemplazan** el conjunto; `Recipe` y
  `RecipeSummary` incluyen sus etiquetas.
- **Filtro en el listado:** `GET /api/recipes?tag=<id|slug>` combinable con `q`,
  `category` y paginación; `total` refleja el conjunto filtrado.
- **UI:** selector de etiquetas con creación al vuelo en `RecipeForm`, etiquetas
  en `RecipeCard` y en el detalle, y filtro del recetario por etiqueta reflejado
  en la URL (query param `tag`), coherente con la búsqueda y la paginación.
- **OpenAPI:** se documentan los nuevos endpoints y campos en el documento
  OpenAPI.

## Impact

- Capabilities:
  - nueva capacidad `recipe-tags` (modelo, migración, CRUD de etiquetas, tags en
    receta y filtro del listado);
  - `recipes` (MODIFY: listado con filtro `tag`; alta/actualización/detalle con
    etiquetas);
  - `recipe-catalog` (MODIFY: filtro por etiqueta en la URL, etiquetas en tarjeta
    y detalle, selector en el formulario);
  - `recipe-search` (MODIFY: índices de rendimiento con `idx_recipe_tags_tag_id`
    y la unicidad de `tags.slug`).
- Backend: `backend/migrations/0007_recipe_tags.sql` (nueva),
  `backend/src/models/tag.rs` (nueva), `backend/src/repo/tags.rs` (nueva),
  `backend/src/routes/tags.rs` (nueva), `backend/src/models/recipe.rs`,
  `backend/src/repo/recipes.rs`, `backend/src/routes/recipes.rs`,
  `backend/src/routes/mod.rs`, `backend/src/routes/openapi.rs` y nuevos tests de
  integración en `backend/tests/`.
- Frontend: `frontend/src/api/types.ts`, `frontend/src/api/client.ts`,
  `frontend/src/api/hooks.ts`, `frontend/src/components/RecipeForm.tsx`,
  `frontend/src/components/RecipeCard.tsx`,
  `frontend/src/pages/RecipeDetailPage.tsx`,
  `frontend/src/pages/RecipesPage.tsx` y tests de hook/componente.
- Sin dependencias nuevas.
- Cierra el issue #55; aplica el ADR-35.
