# Tasks — Comensales habituales y escalado (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Leer `backend/src/routes/tags.rs`, `routes/mod.rs`, `routes/openapi.rs`, `repo/shopping_list.rs` y `tests/schema_constraints.rs` para fijar el patrón de rutas, permisos y tests (verificación: sin cambios de código)
- [x] 1.2 Leer `RecipeDetailPage.tsx`, `SettingsPage.tsx`, `hooks.ts`, `offline/queryPersist.ts` y `lib/shopping.ts` (verificación: identificar puntos exactos de inserción)

## 2. RED (backend)

- [x] 2.1 Test `backend/tests/settings.rs`: `GET /api/settings` devuelve 200 `{default_servings:2}` sin fila previa (verificación: `cargo test --test settings` falla)
- [x] 2.2 Test: `PUT` con 4 devuelve 200 y lectura posterior 4; con 0 y 21 devuelve 422 y no cambia (verificación: falla)
- [x] 2.3 Test: permisos `read:recipes`/`write:recipes` (403 sin permiso) (verificación: falla)

## 3. GREEN (backend)

- [x] 3.1 Migración `0010_household_settings.sql` con CHECK 1..20 y siembra idempotente (verificación: `cargo test --test migrations_fts` y arranque)
- [x] 3.2 `models/settings.rs`, `repo/settings.rs` (lectura con default y `UPDATE`), `routes/settings.rs` (`GET`/`PUT` con `utoipa`), registro en `routes/mod.rs` + tabla de permisos (verificación: tests 2.x en verde)
- [x] 3.3 `routes/openapi.rs`: path y schema; test `backend/tests/openapi.rs` en verde (verificación: `cargo test --test openapi`)
- [x] 3.4 `cargo test --tests`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` (verificación: todo verde)

## 4. RED (frontend)

- [x] 4.1 `lib/scaling.ts` con tests: factor, redondeo (≥10 enteros, 1..10 un decimal, <1 dos decimales), detección de ingredientes sin cantidades (verificación: `npx vitest run src/lib/scaling.test.ts` falla)
- [x] 4.2 `SettingsPage.test.tsx`: carga el valor, guarda 4, muestra error en 422 (verificación: falla)
- [x] 4.3 `RecipeDetailPage.test.tsx`: por defecto usa la preferencia, escala 8→2, selector recalcula, «Ver originales» y aviso de importada (verificación: falla)

## 5. GREEN (frontend)

- [x] 5.1 `api/types.ts` + `api/client.ts` + `hooks.ts`: `getSettings`/`updateSettings`, query `['settings']` persistida (`offline/queryPersist.ts`) (verificación: tests 4.2 en verde)
- [x] 5.2 `SettingsPage.tsx` (+CSS): campo 1..20, guardado explícito, error y confirmación (verificación: tests 4.2 en verde)
- [x] 5.3 `RecipeDetailPage.tsx` (+CSS): selector único de raciones, escalado, aviso y toggle de originales; el alta en la compra usa el valor seleccionado (verificación: tests 4.3 en verde)
- [x] 5.4 `npx vitest run`, `npx tsc --noEmit`, `npm run lint` (verificación: todo verde)

## 6. REFACTOR y cierre

- [x] 6.1 Revisar duplicación entre `lib/scaling.ts` y `backend/src/repo/shopping_list.rs` (documentar por qué no se comparte) (verificación: nota en el PR)
- [x] 6.2 `openspec validate --all --strict` (verificación: 0 errores)
- [x] 6.3 Auditorías `@rust-reviewer` y `@react-reviewer` (verificación: PASS o avisos cerrados)
