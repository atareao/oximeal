# Tasks — Cola de escrituras offline (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Revisar `create_item`/`patch_item` en `backend/src/routes/shopping_list.rs` y el modelo de ítem
- [x] 1.2 Revisar las mutaciones de `frontend/src/api/hooks.ts`

## 2. RED
- [x] 2.1 Test (backend): alta con `client_id` nuevo crea un ítem
- [x] 2.2 Test (backend): alta repetida con el mismo `client_id` no duplica y devuelve el existente
- [x] 2.3 Test (frontend): una mutación sin conexión se encola y se aplica de forma optimista
- [x] 2.4 Test (frontend): al recuperar la conexión la cola se drena en orden
- [x] 2.5 Test (frontend): el toggle de un ítem no sincronizado se aplica a la operación encolada
- [x] 2.6 Ejecutar `cargo test` y `pnpm test`; confirmar el estado RED

## 3. GREEN
- [x] 3.1 Migración `0006_shopping_item_client_id.sql`
- [x] 3.2 `client_id` en modelo y repositorio
- [x] 3.3 Alta idempotente en `POST /api/shopping-list`
- [x] 3.4 `frontend/src/offline/mutationQueue.ts`
- [x] 3.5 Mutaciones offline-aware en `frontend/src/api/hooks.ts`
- [x] 3.6 Indicador de pendientes en `ShoppingPage.tsx`
- [x] 3.7 `cargo test` y `pnpm test` en verde

## 4. REFACTOR
- [x] 4.1 `cargo fmt --check`, `cargo clippy -- -D warnings`, `pnpm typecheck`, `pnpm lint` y `pnpm format:check`
- [x] 4.2 Revisar que un 4xx no recuperable descarta la operación sin bloquear la cola

## 5. Cierre
- [x] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [x] 5.2 Actualizar `PLAN.md` §18.5 y `CHANGELOG.md`; archivar el change
