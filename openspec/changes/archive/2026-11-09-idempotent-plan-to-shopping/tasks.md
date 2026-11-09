# Tasks

## 1. Backend — RED (tests)

- [x] 1.1 Test de idempotencia: `from_plan` del mismo rango dos veces produce la misma lista sin duplicar.
- [x] 1.2 Test de acumulación: `from_plan` del lunes y luego del martes incluye ambos sin duplicar entradas coincidentes.
- [x] 1.3 Test de reescalado: cambiar `servings` de una entrada y re-volcar el mismo rango reescala sin acumular.
- [x] 1.4 Test de solape: `from_plan` de un día y luego de la semana que lo contiene no duplica.
- [x] 1.5 Test «líneas del plan separadas de las manuales»: manual «Tomate» 2 ud + plan «Tomate» 4 ud → dos líneas.
- [x] 1.6 Test de autorización: `from-plan` sin permiso → 401/403; con `write:shopping` → 200.
- [x] 1.7 Test de `consolidate`: fija las fuentes al rango (el volcado previo de otro rango desaparece).
- [x] 1.8 Verificar RED: `cargo test` (los nuevos fallan; el resto no regresiona).

## 2. Backend — GREEN

- [x] 2.1 Migración `backend/migrations/0011_shopping_plan_sources.sql` con la tabla `shopping_plan_sources`.
- [x] 2.2 `rebuild_plan_lines` + helpers de registro (unión) y fijado (reset) de fuentes; retirar `merge_drafts`/`MergeTarget`.
- [x] 2.3 `collect_drafts` → `collect_plan_drafts`: lee la unión de fuentes registradas.
- [x] 2.4 `from_plan` (unión) y `consolidate` (fijar) con el helper compartido.
- [x] 2.5 `agent::set_weekly_plan` fija las fuentes al rango (reset) y reconstruye.
- [x] 2.6 Actualizar doc-comments de rutas y OpenAPI si procede.
- [x] 2.7 Verificar GREEN: `cargo test` y `cargo check`.

## 3. Backend — REFACTOR

- [x] 3.1 Adaptar/renombrar los tests `from_plan_*` existentes que asumían fusión aditiva.
- [x] 3.2 `cargo fmt --check` y `cargo clippy -- -D warnings` sin avisos.
- [x] 3.3 `openspec validate idempotent-plan-to-shopping --strict` en verde.

## 4. Auditoría

- [x] 4.1 Revisión con `@rust-reviewer` (arquitectura, tests, migración, concurrencia).

## 5. Documentación y cierre

- [x] 5.1 Crear `docs/shopping-plan-dump.md` (qué es, ficheros, cómo funciona, decisiones, follow-ups).
- [x] 5.2 Archivar el change (`openspec archive idempotent-plan-to-shopping`) y consolidar la spec.
- [ ] 5.3 PR a `development` (gitmoji), CI en verde y merge.
