# Tasks — Cola de escrituras offline (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Revisar `create_item`/`patch_item` en `backend/src/routes/shopping_list.rs` y el modelo de ítem
- [ ] 1.2 Revisar las mutaciones de `frontend/src/api/hooks.ts`

## 2. RED
- [ ] 2.1 Test (backend): alta con `client_id` nuevo crea un ítem
- [ ] 2.2 Test (backend): alta repetida con el mismo `client_id` no duplica y devuelve el existente
- [ ] 2.3 Test (frontend): una mutación sin conexión se encola y se aplica de forma optimista
- [ ] 2.4 Test (frontend): al recuperar la conexión la cola se drena en orden
- [ ] 2.5 Test (frontend): el toggle de un ítem no sincronizado se aplica a la operación encolada
- [ ] 2.6 Ejecutar `cargo test` y `pnpm test`; confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Migración `0005_shopping_item_client_id.sql`
- [ ] 3.2 `client_id` en modelo y repositorio
- [ ] 3.3 Alta idempotente en `POST /api/shopping-list`
- [ ] 3.4 `frontend/src/offline/mutationQueue.ts`
- [ ] 3.5 Mutaciones offline-aware en `frontend/src/api/hooks.ts`
- [ ] 3.6 Indicador de pendientes en `ShoppingPage.tsx`
- [ ] 3.7 `cargo test` y `pnpm test` en verde

## 4. REFACTOR
- [ ] 4.1 `cargo fmt --check`, `cargo clippy -- -D warnings`, `pnpm typecheck`, `pnpm lint` y `pnpm format:check`
- [ ] 4.2 Revisar que un 4xx no recuperable descarta la operación sin bloquear la cola

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` §18.5 y `CHANGELOG.md`; archivar el change
