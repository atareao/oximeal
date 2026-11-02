# Design — Escalado de la consolidación por raciones

## Context
`collect_drafts(conn, start, end)` ejecuta un `SELECT CAST(r.ingredients AS TEXT)` sobre `meal_plan_entries e JOIN recipes r` y concatena los ingredientes de todas las entradas del rango. No consulta `servings`, así que las cantidades son siempre "por receta", ignorando cuántas raciones pide el plan.

## Decision
1. **Fuente del factor:** `factor = COALESCE(e.servings, 2) / COALESCE(r.servings, 2)` (ambas columnas tienen `DEFAULT 2` y `CHECK (servings IS NULL OR servings >= 1)`). Se leen en el propio `SELECT` para no añadir consultas.
2. **Dónde se aplica:** se multiplica `amount` de cada `IngredientInput` parseado por `factor` **antes** de llamar a `consolidate`. Así el motor de dominio no cambia (sigue agrupando/sumando y redondeando a 2 decimales con `round2`).
3. **Sin ramas por factor 1:** se multiplica siempre (`amount * factor`); `round2` absorbe el ruido de coma flotante.
4. **Familias no aditivas:** multiplicar un `amount = 0.0` (p. ej. "al gusto") deja `0.0`, que el motor sigue tratando como no significativo. Comportamiento preservado.
5. **Granularidad:** el factor es por **entrada de plan** (`meal_plan_entries`), no por receta; si la misma receta se planifica varias veces con raciones distintas, cada ocurrencia escala por su propio factor y luego se suman.

## Contract (SQL)
El default se aplica en SQL (no en Rust), de modo que el `Vec` leído es `(String, i64, i64)`:
```
SELECT CAST(r.ingredients AS TEXT), COALESCE(e.servings, 2), COALESCE(r.servings, 2)
FROM meal_plan_entries e
JOIN recipes r ON r.id = e.recipe_id
WHERE e.date >= ? AND e.date <= ?
```

## Testing
- **Integración** (`backend/tests/shopping_list.rs`): receta `servings=2` con ingrediente 100 g, entrada `servings=4` → 200 g; receta 2 / entrada 2 → 100 g; receta `NULL` / entrada `NULL` → 100 g (factor 1). Y el equivalente por `POST /api/agent/weekly-plan`.
