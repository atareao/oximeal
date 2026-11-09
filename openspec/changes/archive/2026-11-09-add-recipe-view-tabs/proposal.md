# Proposal

## Why

En la vista de detalle de receta (`/recipes/:id`) los ingredientes se muestran antes que las instrucciones, en un único scroll. En recetas largas, el contenido principal (las instrucciones) queda por debajo de una lista de ingredientes potencialmente extensa, y el usuario debe desplazarse para leer los pasos. Separar el detalle en pestañas da acceso inmediato a las instrucciones y deja los ingredientes a un clic.

## What Changes

- Añadir dos pestañas en el detalle de receta: **Instrucciones** (activa por defecto) e **Ingredientes**.
- La pestaña Instrucciones contiene el texto de las instrucciones; si la receta no tiene, se muestra un estado vacío.
- La pestaña Ingredientes contiene el `IngredientEditor` actual (modo lectura y edición en línea), sin cambios funcionales.
- El resto del detalle (cabecera, meta, selector de raciones/escalado, alertas, botonera de acciones, etiquetas y fuente) permanece por encima de las pestañas.
- Las pestañas son accesibles (roles `tablist`/`tab`/`tabpanel` y navegación por teclado).

## Capabilities

### New Capabilities

<!-- Ninguna: se modifica una capacidad existente. -->

### Modified Capabilities

- `recipe-catalog`: el Requirement «Detalle de receta» se amplía con las pestañas Instrucciones/Ingredientes.

## Impact

- Frontend: `frontend/src/pages/RecipeDetailPage.tsx`, `frontend/src/pages/RecipeDetailPage.css`, `frontend/src/components/IngredientEditor.tsx` (sin cambios funcionales) y `frontend/src/pages/RecipeDetailPage.test.tsx`.
- Sin cambios de backend, API ni esquema; sin migraciones.
- Documentación: `docs/recipe-view-tabs.md`; actualización de `openspec/specs/recipe-catalog/spec.md` al archivar.