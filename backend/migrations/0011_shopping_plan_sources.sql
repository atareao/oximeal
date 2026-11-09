-- OxiMeal · Migración 0011: volcado declarativo del plan a la lista de la compra.
--
-- `shopping_plan_sources` es la **fuente declarativa** del volcado del plan: una
-- fila por entrada de `meal_plan_entries` ya volcada. La unidad de fuente es la
-- **entrada** (no el rango), de modo que los solapes (día ⊂ semana) deduplican de
-- forma natural por `entry_id` y el borrado/edición de la entrada hereda por
-- clave foránea (`ON DELETE CASCADE`).
--
-- El endpoint `POST /api/shopping-list/from-plan` hace `INSERT OR IGNORE` de las
-- entradas del rango y reconstruye **todas** las líneas derivadas del plan
-- (`source_plan_id` no nulo) reconsolidando la unión de fuentes registradas, de
-- modo que repetir el volcado es idempotente y editar las raciones reescala sin
-- acumular. `consolidate` fija las fuentes al rango indicado (borra y reinserta).
--
-- La lista de la compra sigue siendo un artefacto reconstruible: no se declara FK
-- en `shopping_list_items`, solo aquí para que borrar una entrada del plan
-- desregistre su fuente en cascada.
--
-- Aditiva y append-only: no edita migraciones previas. Revertirla con
-- `DROP TABLE shopping_plan_sources;` no pierde datos del plan ni de la lista
-- (solo la procedencia del volcado; las líneas del plan se regeneran en el
-- siguiente volcado).
CREATE TABLE IF NOT EXISTS shopping_plan_sources (
    entry_id   TEXT PRIMARY KEY REFERENCES meal_plan_entries(id) ON DELETE CASCADE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
