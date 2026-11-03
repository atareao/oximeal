-- OxiMeal · Migración 0007: etiquetas libres de recetas (relación many-to-many).
--
-- Las etiquetas son un concepto propio, FUERA del índice de texto completo
-- `recipes_fts` (que indexa solo `name` e `instructions`): se consultan con su
-- propio filtro (`GET /api/recipes?tag=<id|slug>`), nunca por búsqueda libre.
-- Mantenerlas separadas evita contaminar las coincidencias de `?q=`.
--
-- El `slug` es la clave canónica: nombre normalizado (espacios recortados y
-- colapsados, minúsculas Unicode, diacríticos conservados). Su restricción
-- `UNIQUE` garantiza la identidad insensible a mayúsculas y a espacios.
--
-- Aditiva: no modifica `recipes` ni `recipes_fts`, por lo que revertirla con
-- `DROP TABLE recipe_tags; DROP TABLE tags;` no pierde datos de recetas.
-- Append-only: no edita migraciones previas.

CREATE TABLE tags (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Asociación many-to-many. Ambas FK borran en cascada: al borrar una receta
-- desaparecen sus asociaciones (la etiqueta permanece en el catálogo) y al
-- borrar una etiqueta desaparecen sus asociaciones (las recetas permanecen).
CREATE TABLE recipe_tags (
    recipe_id  TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    tag_id     TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (recipe_id, tag_id)
);

-- Acelera el filtrado de recetas por etiqueta y el conteo por etiqueta.
CREATE INDEX IF NOT EXISTS idx_recipe_tags_tag_id ON recipe_tags(tag_id);
