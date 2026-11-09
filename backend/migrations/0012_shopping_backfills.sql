-- OxiMeal · Migración 0012: marcador de reconciliaciones de datos de la lista.
--
-- `shopping_backfills` registra las reconciliaciones de datos **de un solo uso**
-- ya aplicadas sobre `shopping_list_items`. Cada fila es un marcador por nombre:
-- su presencia significa que esa reconciliación ya se ejecutó y no debe
-- repetirse en arranques posteriores.
--
-- Se usa una tabla en vez de `PRAGMA user_version` para no acoplarse al marcador
-- del backfill de etiquetas automáticas (`services::autotag`) y poder acumular
-- varias reconciliaciones independientes sin renumerarlas entre sí.
--
-- El nombre canónico del primer marcador es `merge-duplicate-lines` (fusión de
-- las líneas manuales duplicadas heredadas del antiguo `from-plan` aditivo).
--
-- Aditiva y append-only: no edita migraciones previas. Revertirla con
-- `DROP TABLE shopping_backfills;` solo provoca que las reconciliaciones se
-- reintenten en el siguiente arranque (son idempotentes).
CREATE TABLE IF NOT EXISTS shopping_backfills (
    name       TEXT PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
