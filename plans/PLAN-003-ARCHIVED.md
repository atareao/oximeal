# PLAN-003 — Ingredientes editables

## Objetivo
Corregir los ingredientes de las recetas: (1) desglosar cantidad y unidad al importar, y (2) permitir editarlos directamente en el detalle.

## Metodología
SDD (OpenSpec) + TDD. Change: `openspec/changes/editable-ingredients/`. Aprobación del usuario antes de escribir código.

## Alcance
- Backend: parseo de `recipeIngredient` en la importación (capacidad `recipes`).
- Frontend: edición inline de ingredientes en el detalle (capacidad `recipe-catalog`).

## Tareas
Ver `openspec/changes/editable-ingredients/tasks.md`.

## Cierre
Al completar: `openspec archive editable-ingredients`, `docs/editable-ingredients.md` y renombrar este plan a `plans/PLAN-003-ARCHIVED.md`.
