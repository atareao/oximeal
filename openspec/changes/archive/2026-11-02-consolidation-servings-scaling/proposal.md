# Change: Escalado de la consolidación por raciones

## Why
`repo::shopping_list::collect_drafts` recolecta los ingredientes de todas las recetas del rango ignorando las raciones: una receta definida para 2 raciones planificada a 4 no duplica las cantidades (hallazgo M7 de la auditoría `sqlite-expert`, issue #3).

## What Changes
- `collect_drafts` SHALL leer `meal_plan_entries.servings` (raciones planificadas) y `recipes.servings` (raciones de la receta) y SHALL escalar cada ingrediente por el factor `raciones_planificadas / raciones_receta` antes de consolidar.
- Cuando cualquiera de las dos sea `NULL`, SHALL usarse el valor por defecto del esquema (`2`), de modo que `NULL`/`NULL` y `2`/`2` dan factor `1`.

## Impact
- Capability: `ingredient-consolidation`.
- Código: `backend/src/repo/shopping_list.rs` (`collect_drafts`).
- Afecta a `POST /api/shopping-list/consolidate` y a `POST /api/agent/weekly-plan` (ambos reconstruyen la lista vía `collect_drafts`).
- No cambia el contrato de la API ni la firma del motor de dominio `consolidate` (se escala la entrada `IngredientInput` antes de consolidar).
