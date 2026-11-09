# PLAN-009 — Reconciliación de duplicados de la lista de la compra y orden determinista

**Issue:** #131 · **Change OpenSpec:** `dedupe-shopping-items` · **Rama prevista:** `feature/dedupe-shopping-items`

## Objetivo

Eliminar los duplicados heredados de la lista de la compra —líneas **manuales** con la misma clave creadas por el antiguo `from-plan` aditivo— mediante una reconciliación de arranque idempotente, y garantizar/ordenar de forma determinista el listado. Sin cambios de contrato HTTP ni de frontend.

## Contexto

- El antiguo `from-plan` (aditivo, feature #123) insertaba las líneas del plan como **manuales** (`source_plan_id = NULL`). Repetir el volcado dejaba filas duplicadas con la misma clave, indistinguibles de los ítems manuales.
- #128/#129 corrigió el comportamiento hacia adelante (declarativo e idempotente) pero **no limpia** los duplicados ya existentes.
- El follow-up de la auditoría de #124 pedía orden determinista; el `ORDER BY` no determinista vivía en `merge_drafts`, ya retirado. Hay que garantizarlo y testearlo.
- Patrón del repo para datos heredados: backfill de arranque con marcador persistente (`services::autotag::backfill`).

## Decisiones

- **D1.** Reconciliación de arranque con marcador en tabla nueva `shopping_backfills(name PK, applied_at)`; no se usa `PRAGMA user_version` para no acoplarse al autotag. No se hace en migración SQL porque la clave requiere canonicalizar unidades.
- **D2.** Clave de fusión: `(nombre normalizado, familia de unidad, unidad de presentación canónica)`. Solo líneas manuales.
- **D3.** Supervivente: la de `id` menor; conserva su `name` y `category`.
- **D4.** Suma en unidades base (`units::canonicalize`, redondeo a 2 decimales), expresada en la unidad del supervivente; familias no aditivas (`ToTaste`) → `0.0`.
- **D5.** `checked` = `1` si alguna lo estaba.
- **D6.** El supervivente conserva su `client_id`; no hay conflicto con el índice único parcial porque las demás filas se borran.
- **D7.** Orígenes distintos (manual / receta / plan) no se fusionan entre sí.

## Alcance

### Backend
- `backend/migrations/0012_shopping_backfills.sql` — tabla `shopping_backfills`.
- `backend/src/services/shopping_reconcile.rs` — servicio nuevo con `reconcile(pool)` y marcador.
- `backend/src/services/mod.rs` — registrar el módulo.
- `backend/src/main.rs` — llamar al servicio tras el backfill de autotag, tolerante a fallos (no impide el arranque).
- `backend/tests/shopping_reconcile.rs` — tests de reconciliación.
- `backend/tests/shopping_list.rs` — test de orden determinista.
- `backend/tests/migrations_fts.rs` — añadir `shopping_backfills` a las tablas esperadas.

### Frontend
- Ninguno.

## Tareas TDD

### Fase RED
- [x] T1. Test: dos líneas manuales «Tomate» 2 ud y 3 ud → tras reconciliar, una de 5 ud.
- [x] T2. Test: «Harina» «1 kg» + «500 g» → una línea equivalente a 1500 g.
- [x] T3. Test: «Tomate» «2 ud» + «500 g» → dos líneas (familias distintas).
- [x] T4. Test: «Tomate» manual 2 ud + «Tomate» del plan 4 ud → dos líneas (orígenes distintos).
- [x] T5. Test: supervivente = `id` menor y `checked = 1` si alguna estaba marcada.
- [x] T6. Test: idempotencia (segunda ejecución no cambia nada) y marca persistente escrita.
- [x] T7. Test: orden determinista del listado `GET /api/shopping-list` con nombres de distinta capitalización.
- [x] T8. Verificar RED (`cargo test`).

### Fase GREEN
- [x] T9. Migración `0012_shopping_backfills.sql`.
- [x] T10. `services::shopping_reconcile::reconcile(pool)` (transacción `BEGIN IMMEDIATE`, marcador).
- [x] T11. Llamada en `main.rs` tras autotag (log de error sin abortar).
- [x] T12. Verificar GREEN (`cargo test`).

### Fase REFACTOR
- [x] T13. `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`.
- [x] T14. `openspec validate dedupe-shopping-items --strict`.

### Auditoría y cierre
- [x] T15. Auditoría con `@rust-reviewer`.
- [x] T16. `docs/shopping-list-dedupe.md`.
- [x] T17. Archivar el change y consolidar `openspec/specs/shopping-list/spec.md`.
- [ ] T18. PR a `development` (gitmoji), CI en verde y merge; marcar issue #131.

## Criterios de aceptación (DoD)

- La reconciliación fusiona solo líneas manuales con la misma clave, es determinista e idempotente (marcador).
- Orígenes distintos y unidades incompatibles permanecen separados.
- El listado tiene orden estable y está cubierto por un test.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` y `openspec validate --all --strict` en verde.
- Contrato HTTP y frontend sin cambios.

## Riesgos

- **Fusión de líneas iguales intencionadas**: aceptado (no hay clave de separación en el modelo).
- **Cambio de cantidad del supervivente** al sumar: documentado.
- **Fallo de la reconciliación**: no se escribe el marcador y se reintenta en el siguiente arranque; el arranque continúa.
- Los 2 tests `config::*` fallan en local por el `.env` raíz (preexistente, ajeno).

## Verificación final

- `cargo test --test shopping_reconcile`, `--test shopping_list`, `--test migrations_fts` y el resto de la suite.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- `openspec validate --all --strict`.