# Proposal

## Why

Las recetas importadas por URL llegan con los ingredientes sin desglosar: el mapeo de JSON-LD guarda la línea `recipeIngredient` completa como `name` y fija `amount = 1`, `unit = "ud"`. Así, «40 g de queso parmesano» queda como nombre completo + «1 ud», lo que rompe el escalado por raciones y obliga a corregirlo a mano. Además, corregir un ingrediente hoy exige abrir el modal completo de edición y tocar también el resto de campos.

## What Changes

- **Backend (import)**: interpretar cada `recipeIngredient` al importar para separar cantidad, unidad y nombre; recaer en `amount = 1` / `unit = "ud"` cuando no se detecte cantidad. No cambia la API ni el esquema SQLite.
- **Frontend (detalle)**: permitir editar los ingredientes (nombre/cantidad/unidad) directamente en el detalle de la receta, sin abrir el modal, persistiendo con `PUT /api/recipes/{id}`.

## Capabilities

### New Capabilities
<!-- Ninguna: se modifican capacidades existentes. -->

### Modified Capabilities
- `recipes`: la importación desglosa cantidad y unidad de cada ingrediente.
- `recipe-catalog`: la edición desde la UI incorpora la edición de ingredientes en el propio detalle.

## Impact

- Backend: `backend/src/domain/schema_org.rs` (mapeo de `recipeIngredient`) y un helper de parseo nuevo con tests; el test que fijaba el comportamiento antiguo se actualiza.
- Frontend: `frontend/src/pages/RecipeDetailPage.tsx`, edición inline reutilizando `useUpdateRecipe`; sin cambios en `frontend/src/api/types.ts`.
- Sin cambios de contrato de API, permisos ni esquema de datos (los ingredientes ya son JSON con `name`/`amount`/`unit`).
