-- OxiMeal · Migración 0009: procedencia de etiquetas y supresión de sugerencias.
--
-- Introduce el etiquetado automático de recetas (capacidad `recipe-tags`):
--
--   * `recipe_tags.source` registra la procedencia de cada asociación:
--     `manual` (la fijó el usuario) o `auto` (la sugirió la heurística). La
--     manual tiene precedencia: una etiqueta presente en ambas formas se guarda
--     una sola vez como `manual`. Las asociaciones preexistentes quedan como
--     `manual` gracias al `DEFAULT`.
--   * `recipe_tag_suppressions` permite retirar por receta una sugerencia
--     automática sin que vuelva a asignarse. Clave primaria compuesta y FK en
--     cascada: al borrar la receta o la etiqueta desaparece su supresión.
--   * El catálogo automático (`vegetariano`, `vegano`, `sin-gluten`,
--     `sin-lactosa`, `picante`) se siembra de forma idempotente por `slug`,
--     aunque ninguna receta lo use, para que pueda filtrarse como cualquier
--     etiqueta libre.
--
-- Aditiva y append-only: no edita migraciones previas ni el índice FTS.
-- Revertirla con `DROP TABLE recipe_tag_suppressions;` y
-- `ALTER TABLE recipe_tags DROP COLUMN source;` no pierde recetas; las etiquetas
-- manuales permanecen.

ALTER TABLE recipe_tags ADD COLUMN source TEXT NOT NULL DEFAULT 'manual'
    CHECK (source IN ('manual', 'auto'));

-- Supresión por receta de una etiqueta automática.
CREATE TABLE IF NOT EXISTS recipe_tag_suppressions (
    recipe_id  TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    tag_id     TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (recipe_id, tag_id)
);

-- Acelera el conteo de supresiones por etiqueta.
CREATE INDEX IF NOT EXISTS idx_recipe_tag_suppressions_tag_id
    ON recipe_tag_suppressions(tag_id);

-- Catálogo automático: idempotente por `slug` (UNIQUE). El id se genera con
-- `randomblob` porque SQLite no expone una función UUID nativa.
INSERT INTO tags (id, name, slug) VALUES
    (lower(hex(randomblob(16))), 'Vegetariano', 'vegetariano'),
    (lower(hex(randomblob(16))), 'Vegano', 'vegano'),
    (lower(hex(randomblob(16))), 'Sin gluten', 'sin-gluten'),
    (lower(hex(randomblob(16))), 'Sin lactosa', 'sin-lactosa'),
    (lower(hex(randomblob(16))), 'Picante', 'picante')
ON CONFLICT(slug) DO NOTHING;
