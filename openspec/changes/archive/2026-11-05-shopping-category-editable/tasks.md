# Tareas — shopping-category-editable

## 1. RED

- [x] 1.1 Unitarios de `categorize` para las keywords nuevas y las colisiones (`morcilla`, `higado`, `limonada`, `manzanilla`, `tomate frito`) en `backend/src/domain/category.rs`.
- [x] 1.2 Integración: PATCH con `category` válida actualiza el ítem (`backend/tests/shopping_list.rs`).
- [x] 1.3 Integración: PATCH con `category` desconocida devuelve 400 y no modifica.
- [x] 1.4 Frontend: test del selector de categoría en `ShoppingRow`/`ShoppingPage`.
- [x] 1.5 Ejecutar `cargo test` y `npx vitest run`; confirmar rojo por la causa esperada.

## 2. GREEN

- [x] 2.1 Ampliar `RULES` con frutas y verduras (Verdulería) y productos habituales del resto de pasillos, ordenadas de específico a genérico.
- [x] 2.2 `ShoppingItemPatch.category: Option<String>` + validación contra `CATEGORY_ORDER`.
- [x] 2.3 `patch_item` actualiza `category` cuando viene.
- [x] 2.4 `ShoppingItemPatch` en TS + mutación de patch que acepte `category`.
- [x] 2.5 Selector accesible en `ShoppingRow` conectado a la mutación.
- [x] 2.6 `cargo test`, `npx vitest run` y `npx tsc --noEmit` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt`, `clippy -D warnings` y `npm run lint`.
- [x] 3.2 Revisar duplicación de la lista de categorías entre backend y frontend.
- [x] 3.3 `just ci` y `just ci-ui`.

## 4. Cierre

- [x] 4.1 `openspec validate --all --strict`.
- [x] 4.2 Archivar el change.
- [x] 4.3 CHANGELOG y PR a `development`.