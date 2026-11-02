# Design

## Context

`backend/src/db.rs` configura el pool con `journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5s` y `max_connections=5`. Hay tres transacciones de escritura que combinan lectura y escritura dentro de la misma transacción: `repo::shopping_list::consolidate`, `repo::agent::set_weekly_plan` y `repo::agent::add_shopping_items`; todas abren con `pool.begin()` (BEGIN diferido). El esquema vive en migraciones append-only (`0001`…`0004`) y `recipes` tiene un índice FTS5 external-content (`recipes_fts`) con triggers `recipes_ai/ad/au`.

## Goals / Non-Goals

**Goals:**
- Eliminar la posibilidad de `SQLITE_BUSY_SNAPSHOT` en escrituras concurrentes sin cambiar el contrato HTTP.
- Hacer explícita la integridad referencial y añadir defensa en profundidad con `CHECK`.
- Preservar íntegros los datos existentes, los triggers de FTS5 y los índices.

**Non-Goals:**
- Añadir el endpoint de borrado de recetas (se documenta la política `ON DELETE` para cuando exista).
- Cambiar `busy_timeout`, `max_connections` o el modo de journal.
- Migrar a `sqlx` offline mode o reescribir consultas existentes.

## Decisions

- **`BEGIN IMMEDIATE` vía `SqlitePool::begin_with("BEGIN IMMEDIATE")`** (API pública de sqlx 0.9; `&'static str` implementa `SqlSafeStr`). Alternativa descartada: gestionar `BEGIN IMMEDIATE`/`COMMIT`/`ROLLBACK` a mano sobre una `PoolConnection`, que obliga a RAII manual y no aporta ventaja.
- **`ON DELETE RESTRICT`** en `meal_plan_entries.recipe_id`. Alternativa descartada: `CASCADE`, que borraría silenciosamente entradas del plan al borrar una receta; `RESTRICT` hace visible el conflicto y obliga a una acción explícita.
- **Reconstrucción de tabla (12 pasos)** para añadir `CHECK`, que SQLite no admite como `ALTER TABLE … ADD CONSTRAINT`. Se usa el marcador `-- no-transaction` de sqlx para poder ejecutar `PRAGMA foreign_keys=OFF` (prohibido dentro de una transacción), reconstruir las tablas, recrear los triggers FTS5 y reindexar con `INSERT INTO recipes_fts(recipes_fts) VALUES('rebuild')`, cerrando con `PRAGMA foreign_key_check`. Alternativas descartadas: `PRAGMA writable_schema` (corrompe el esquema, no soportado) y no añadir los `CHECK` en `recipes` (dejaría la petición de la issue a medias).
- **Orden de reconstrucción**: `recipes` primero, luego `meal_plan_entries` (que la referencia) y por último `shopping_list_items`; con `foreign_keys` desactivado durante el script.

## Risks / Trade-offs

- La migración es más pesada que un `ALTER TABLE`; se mitiga con `-- no-transaction` + `foreign_key_check` y un test que verifica que los datos y los triggers de FTS5 sobreviven.
- Reconstruir `recipes` debe reindexar `recipes_fts`; se cubre con test de búsqueda tras migrar.
- El test de reproducción de `SQLITE_BUSY_SNAPSHOT` es sensible a la concurrencia; se diseña determinista (dos conexiones con intercalado explícito) en lugar de por estrés.
