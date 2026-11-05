# Categoría editable en la lista de la compra

## Why

La categoría de un ítem se decide con una tabla estática de palabras clave por subcadena (`backend/src/domain/category.rs`). Cualquier producto que no esté en la tabla cae en `Otros` y no hay forma de corregirlo: `PATCH /api/shopping-list/{id}` solo acepta `checked` y `amount`, y la UI no expone la categoría. Caso real: `melocotón` acaba en `Otros` porque `melocoton` no está en la tabla y `melon` no es subcadena suya.

## What Changes

- Ampliar `RULES` con frutas, verduras y productos habituales que hoy faltan, manteniendo el orden de más específico a más genérico para evitar colisiones de subcadena (`morcilla` antes que `mora`, `higado` antes que `higo`, `limonada` antes que `limon`, `manzanilla` antes que `manzana`).
- Añadir `category` a `ShoppingItemPatch` y al `UPDATE` de `patch_item`, con validación contra las 8 categorías canónicas (400 si no es válida).
- Exponer un selector de categoría por ítem en la UI (`ShoppingRow`), que persiste con el PATCH existente.
- Tests: unitarios de `categorize` para las keywords nuevas y las colisiones, integración del PATCH de categoría (válida e inválida) y test de UI del selector.

## Impact

- Specs: `shopping-list` (MODIFIED «Toggle y edición», ADDED «Categorías canónicas»).
- Código: `backend/src/domain/category.rs`, `backend/src/models/shopping_list.rs`, `backend/src/repo/shopping_list.rs`, `backend/src/routes/shopping_list.rs`, `frontend/src/components/ShoppingRow.tsx`, `frontend/src/pages/ShoppingPage.tsx`, `frontend/src/api/types.ts`, `frontend/src/api/hooks.ts`.
- Sin migración de BD: `category` ya es `TEXT NOT NULL DEFAULT 'Otros'`.
- Sin cambios incompatibles: `category` es opcional en el PATCH.