-- OxiMeal · Migración 0008: volcado de una receta a la lista de la compra.
--
-- Introduce la procedencia de las líneas de la lista: una línea puede venir de
-- forma manual (`source_plan_id` y `source_recipe_id` nulos), del plan semanal
-- (`source_plan_id` no nulo) o del volcado de una o varias recetas
-- (`source_recipe_id` no nulo).
--
-- `shopping_recipe_sources` es la **fuente declarativa** del volcado: registra
-- las recetas volcadas y sus raciones solicitadas. El endpoint
-- `POST /api/shopping-list/from-recipe` hace upsert aquí y reconstruye en su
-- totalidad las líneas derivadas de recetas (`source_recipe_id` no nulo)
-- reconsolidando todas las fuentes registradas, de modo que repetir la petición
-- es idempotente y cambiar las raciones reescala sin acumular.
--
-- Cuando una línea fusiona ingredientes de **varias** recetas (mismo nombre
-- canónico y familia/unidad), se marca con `source_recipe_id = 'consolidated'`
-- en lugar del id de una receta concreta: la atribución ya no es unívoca.
--
-- `source_recipe_id` es NULL en las líneas manuales y en las del plan, y **no
-- NULL** en las derivadas de recetas. No declara FK: la lista es un artefacto
-- reconstruible, así que borrar una receta desregistra su fuente (cascade sobre
-- `shopping_recipe_sources`) y sus líneas se depuran en el siguiente volcado.
--
-- Aditiva y append-only: no edita migraciones previas. Revertirla con
-- `DROP TABLE shopping_recipe_sources;` y el `DROP COLUMN source_recipe_id` no
-- pierde datos de recetas ni de la lista (solo la procedencia del volcado).

ALTER TABLE shopping_list_items ADD COLUMN source_recipe_id TEXT;

CREATE INDEX IF NOT EXISTS idx_shopping_source_recipe_id
    ON shopping_list_items (source_recipe_id);

-- Fuente declarativa del volcado: receta volcada y raciones solicitadas.
-- Borrar la receta desregistra su fuente en cascada (las líneas derivadas se
-- depuran en el siguiente volcado).
CREATE TABLE IF NOT EXISTS shopping_recipe_sources (
    recipe_id   TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
    servings    INTEGER NOT NULL CHECK (servings >= 1),
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
