# Tasks — Hardening de SQLite (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Confirmar líneas base: `cargo test -p oximeal-backend` en verde y `openspec validate --all --strict`
- [x] 1.2 Identificar los tres puntos de escritura transaccional y el patrón read-then-write

## 2. Transacciones IMMEDIATE (issue #2)
- [x] 2.1 RED: test determinista que reproduce `SQLITE_BUSY_SNAPSHOT` con `BEGIN` diferido (dos conexiones con intercalado)
- [x] 2.2 RED: test de escrituras concurrentes (`add_shopping_items`/`consolidate`/`set_weekly_plan`) sin `BUSY_SNAPSHOT`
- [x] 2.3 GREEN: abrir las tres transacciones con `SqlitePool::begin_with("BEGIN IMMEDIATE")`
- [x] 2.4 REFACTOR: helper/comentario único y documentación del criterio (lecturas con `begin()`, escrituras con `begin_with`)

## 3. Restricciones de esquema (issue #4)
- [x] 3.1 RED: test de que `ON DELETE RESTRICT` impide borrar una receta referenciada por el plan
- [x] 3.2 RED: tests de `CHECK` (amount < 0, servings < 1, prep_time_minutes < 0 rechazados; válidos y NULL aceptados)
- [x] 3.3 RED: test de que datos y búsqueda FTS5 (`recipes_fts`) sobreviven a la migración
- [x] 3.4 GREEN: migración `0005_schema_constraints.sql` (12 pasos, `-- no-transaction`, rebuild de las tres tablas, recreación de triggers y reindexado)
- [x] 3.5 REFACTOR: verificar `foreign_key_check`, índices y que la migración es idempotente en arranque limpio

## 4. Cierre
- [x] 4.1 `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test -p oximeal-backend` y `--features embed-ui`
- [x] 4.2 `openspec validate sqlite-hardening --strict` y `openspec validate --all --strict`
- [ ] 4.3 Actualizar `PLAN.md` y cerrar las issues #2 y #4 (tras merge)
