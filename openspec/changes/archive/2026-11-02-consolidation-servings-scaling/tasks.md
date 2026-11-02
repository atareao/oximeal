# Tasks — Escalado de la consolidación por raciones (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Confirmar línea base: `cargo test -p oximeal-backend` en verde y `openspec validate --all --strict`
- [x] 1.2 Localizar `collect_drafts` en `backend/src/repo/shopping_list.rs` y los tests de consolidación en `backend/tests/shopping_list.rs` y `backend/tests/agent.rs`

## 2. Escalado por raciones (issue #3)
- [x] 2.1 RED: test de que una receta `servings=2` con 100 g planificada a `servings=4` produce 200 g
- [x] 2.2 RED: test de que `servings=2`/`servings=2` no altera las cantidades
- [x] 2.3 RED: test de que `NULL`/`NULL` usa el factor 1
- [x] 2.4 GREEN: `collect_drafts` lee `e.servings` y `r.servings` y escala `IngredientInput.amount`
- [x] 2.5 REFACTOR: extraer el cálculo del factor a un helper puro y documentarlo

## 3. Cierre
- [x] 3.1 `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test -p oximeal-backend` (con y sin `embed-ui`)
- [x] 3.2 `openspec validate consolidation-servings-scaling --strict` y `openspec validate --all --strict`
- [x] 3.3 Actualizar `PLAN.md` y cerrar la issue #3 (tras merge)
