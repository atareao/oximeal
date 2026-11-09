# Plan 008 — Volcado declarativo e idempotente del plan a la lista de la compra

- **Issue:** #128
- **Change OpenSpec:** `idempotent-plan-to-shopping`
- **Capacidad afectada:** `shopping-list` (Requirement «Volcado aditivo desde el plan» → «Volcado declarativo e idempotente desde el plan»; «Consolidación desde el plan» matizada)
- **Rama:** `feature/idempotent-plan-to-shopping` (desde `development`)
- **Fecha:** 2026-11-09

## 1. Objetivo

Hacer **declarativo e idempotente** el volcado del plan a la lista (`POST /api/shopping-list/from-plan`), replicando el patrón ya usado por `from-recipe`:

1. Registrar como fuentes las **entradas del plan** volcadas, en la tabla `shopping_plan_sources`.
2. Reconstruir las líneas derivadas del plan (`source_plan_id` no nulo) desde la **unión** de fuentes registradas.
3. Repetir el mismo rango NO duplica; añadir otro rango acumula; editar el plan y re-volcar **reescala sin acumular**; los solapes (día ⊂ semana) deduplican por entrada.
4. Unificar el mecanismo con `POST /api/shopping-list/consolidate` (que pasa a **fijar** las fuentes al rango) y con `agent::set_weekly_plan`.

## 2. Mapa arquitectónico

| Elemento | Fichero | Rol |
|---|---|---|
| Migración | `backend/migrations/0011_shopping_plan_sources.sql` | Nueva tabla `shopping_plan_sources` |
| Repositorio | `backend/src/repo/shopping_list.rs` | `from_plan`, `consolidate`, `collect_drafts`, `replace_consolidated`, `merge_drafts` |
| Agente | `backend/src/repo/agent.rs` | `set_weekly_plan` (reconstrucción de la compra, líneas ~123-127) |
| Rutas | `backend/src/routes/shopping_list.rs` | Handlers `from_plan` (204-240) y `consolidate` (133-161) |
| Spec | `openspec/specs/shopping-list/spec.md` | Requirements del volcado y de la consolidación |
| Tests | `backend/tests/shopping_list.rs` | Tests `from_plan_*` (636-1040) y `consolidLiate_*` (325-634) |
| Tests transacciones | `backend/tests/write_transactions.rs` | `consolidate` en rango (175) |
| Frontend | `frontend/src/api/client.ts` | `addPlanToShoppingList` (sin cambios de contrato) |

**Esquema actual relevante:** `shopping_list_items(id, name, amount, unit, category, checked, source_plan_id, source_recipe_id, updated_at, client_id)`; `shopping_recipe_sources(recipe_id, servings)`; `meal_plan_entries(id, date, meal_type, recipe_id, role, servings)`.

## 3. Decisiones de diseño

- **D1 — Fuente = entrada del plan.** `shopping_plan_sources(entry_id TEXT PRIMARY KEY REFERENCES meal_plan_entries(id) ON DELETE CASCADE, created_at)`. Deduplica solapes y hereda el borrado/edición por clave foránea. Alternativa descartada: registrar rangos.
- **D2 — Reconstrucción única.** `rebuild_plan_lines(conn)`: borra las líneas con `source_plan_id IS NOT NULL` e inserta el consolidado de la unión de fuentes. Se retiran `merge_drafts`/`MergeTarget`; `replace_consolidated` se sustituye por este helper.
- **D3 — `from_plan` = unión.** `INSERT OR IGNORE` de las entradas del rango en fuentes + reconstrucción.
- **D4 — `consolidate` = fijar.** Borra todas las fuentes, inserta las del rango + reconstrucción (conserva la semántica de «reemplazar»).
- **D5 — `agent::set_weekly_plan` = unión.** Tras reescribir las entradas, `INSERT OR IGNORE` del rango + reconstrucción.
- **D6 — Líneas del plan atribuidas.** `source_plan_id = 'consolidated'`; no se fusionan con manuales ni con líneas de receta (cambio visible; ver riesgos).

## 4. Tareas (TDD)

### 4.1 Backend — RED

- [x] T1. Test de idempotencia: `from_plan` del mismo rango dos veces → misma lista, sin duplicar.
- [x] T2. Test de acumulación: `from_plan` del lunes y luego del martes → aparecen ambos, sin duplicar entradas coincidentes.
- [x] T3. Test de reescalado: cambiar `servings` de una entrada y re-volcar → reescala sin acumular.
- [x] T4. Test de solape: `from_plan` de un día y luego de la semana que lo contiene → sin duplicar.
- [x] T5. Test «líneas del plan separadas de las manuales» (manual «Tomate» 2 ud + plan «Tomate» 4 ud → dos líneas).
- [x] T6. Test de autorización: `from-plan` sin permiso → 401/403; con `write:shopping` → 200.
- [x] T7. Test de `consolidate`: fija las fuentes al rango (el volcado previo de otro rango desaparece).
- [x] T8. Verificar RED: `cargo test`.

### 4.2 Backend — GREEN

- [x] T9. Migración `0011_shopping_plan_sources.sql`.
- [x] T10. `rebuild_plan_lines` + helpers de registro (unión) y fijado (reset) de fuentes; retirar `merge_drafts`/`MergeTarget`.
- [x] T11. Adaptar `collect_drafts` para leer la unión de fuentes.
- [x] T12. `from_plan` (unión) y `consolidate` (fijar) con el helper compartido.
- [x] T13. agent::set_weekly_plan fija las fuentes al rango (reset) y reconstruye.
- [x] T14. Actualizar doc-comments de rutas y OpenAPI si procede.
- [x] T15. Verificar GREEN: `cargo test` y `cargo check`.

### 4.3 Backend — REFACTOR

- [x] T16. Adaptar/renombrar los tests `from_plan_*` existentes que asumían fusión aditiva.
- [x] T17. `cargo fmt --check` y `cargo clippy -- -D warnings` sin avisos.
- [x] T18. `openspec validate idempotent-plan-to-shopping --strict` en verde.

### 4.4 Auditoría

- [x] T19. Revisión con `@rust-reviewer` (arquitectura, tests, migración, concurrencia).

### 4.5 Documentación y cierre

- [x] T20. Crear `docs/shopping-plan-dump.md` (qué es, ficheros, cómo funciona, decisiones, follow-ups).
- [x] T21. Archivar el change (`openspec archive idempotent-plan-to-shopping`) y consolidar la spec.
- [ ] T22. PR a `development` (gitmoji), CI en verde y merge.

## 5. Criterios de aceptación

- `from-plan` idempotente y reescalable; `consolidate` fija las fuentes; ambos comparten la reconstrucción.
- Suite backend 100% verde (incluidos los tests existentes adaptados) y `clippy` sin warnings.
- `openspec validate idempotent-plan-to-shopping --strict` en verde.
- PR a `development` con CI en verde.

## 6. Riesgos

- **Cambio de comportamiento visible (BREAKING):** `from-plan` deja de sumar sobre ítems manuales; las líneas del plan quedan atribuidas y separadas. Aceptado en el diseño (opción A).
- **Datos heredados:** las líneas del plan se regeneran en el primer volcado/consolidación; solo hay migración de esquema.
- **`agent::set_weekly_plan`** cambia de «reset» a «unión»: revisar sus tests existentes.
