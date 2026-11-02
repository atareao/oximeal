# Tasks — Semántica de búsqueda trigram (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Confirmar línea base: `cargo test -p oximeal-backend` en verde y `openspec validate --all --strict`
- [x] 1.2 Revisar `repo::recipes::list` y los tests de búsqueda en `backend/tests/recipes.rs`

## 2. Caracterización y documentación (issue #6)
- [x] 2.1 RED/characterization: test de subcadena contigua multi-palabra devuelve la receta
- [x] 2.2 RED/characterization: test de que palabras no contiguas no coinciden (`total = 0`)
- [x] 2.3 RED/characterization: frontera de 3 chars usa FTS (término solo en instrucciones) y `<3` usa `LIKE` solo sobre el nombre
- [x] 2.4 Docstring de `repo::recipes::list` con la semántica real
- [x] 2.5 Doc: crear `docs/search.md` con la expectativa de precisión y el fallback `<3`

## 3. Cierre
- [x] 3.1 `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test -p oximeal-backend` (con y sin `embed-ui`)
- [x] 3.2 `openspec validate trigram-search-semantics --strict` y `openspec validate --all --strict`
- [x] 3.3 Actualizar `PLAN.md` y cerrar la issue #6 (tras merge)
