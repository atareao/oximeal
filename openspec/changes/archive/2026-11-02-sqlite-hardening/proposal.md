# Change: Hardening de SQLite — transacciones de escritura y restricciones de esquema

## Why
Quedan abiertos dos hallazgos de la auditoría `sqlite-expert` (issues #2 y #4). Por un lado, las transacciones de escritura usan `BEGIN` diferido, de modo que un patrón *read-then-write* concurrente en modo WAL puede fallar con `SQLITE_BUSY_SNAPSHOT` (que `busy_timeout` no resuelve). Por otro, el esquema carece de `ON DELETE` explícito en `meal_plan_entries.recipe_id` y de restricciones `CHECK` en `amount`/`servings`/`prep_time_minutes`, así que la integridad depende únicamente de la validación en las rutas.

## What Changes
- Abrir las transacciones de escritura con `BEGIN IMMEDIATE` (vía `SqlitePool::begin_with`) en `repo::shopping_list::consolidate`, `repo::agent::set_weekly_plan` y `repo::agent::add_shopping_items`.
- Declarar `ON DELETE RESTRICT` en la clave foránea `meal_plan_entries.recipe_id → recipes(id)` (explícita; mismo efecto que el `NO ACTION` por defecto, pero estable y documentada ante el futuro endpoint de borrado de recetas).
- Añadir `CHECK` de dominio: `shopping_list_items.amount >= 0`; `recipes.servings` y `meal_plan_entries.servings` NULL o `>= 1`; `recipes.prep_time_minutes` NULL o `>= 0`.
- Nueva migración append-only `backend/migrations/0005_schema_constraints.sql` que reconstruye las tres tablas con el procedimiento de 12 pasos, preservando datos, los triggers de FTS5 y los índices.

## Capabilities

### New Capabilities
- `persistence`: comportamiento transaccional de escritura y garantías de integridad del esquema SQLite (transacciones `IMMEDIATE`, claves foráneas explícitas y restricciones `CHECK`).

### Modified Capabilities

## Impact
- Backend Rust (`backend/src/repo/*`, `backend/src/db.rs`) y una migración nueva; sin cambios de contrato HTTP ni de frontend.
- Comportamiento observable: las escrituras concurrentes dejan de poder fallar con `SQLITE_BUSY_SNAPSHOT`; la base de datos rechaza cantidades, raciones o tiempos inválidos incluso si llegan por una vía que no pase por la validación de rutas.
- Los datos existentes se preservan (rebuild con copia); `foreign_keys` se desactiva temporalmente durante la migración mediante el marcador `-- no-transaction` de sqlx.
