===INICIO===
# recipe-view-tabs — Pestañas en el detalle de receta

**Issue:** #125 · **Change OpenSpec:** `add-recipe-view-tabs` (archivado) · **Plan:** `plans/PLAN-007-ARCHIVED.md`

## Qué es

La vista de detalle de receta (`/recipes/:id`) organiza el contenido en dos pestañas:

1. **Instrucciones** — pestaña principal, activa por defecto.
2. **Ingredientes** — segunda pestaña, con el editor de ingredientes.

El resto del detalle (cabecera con nombre y categoría, meta de tiempo/raciones, selector de raciones y escalado, alertas, botonera editar/borrar/añadir-a-lista, tipo de dieta, etiquetas y enlace a la fuente original) se muestra siempre por encima de las pestañas.

## Ficheros implicados

- `frontend/src/pages/RecipeDetailPage.tsx` — `Tabs` de antd con `items` y `defaultActiveKey="instructions"`; incluye el estado vacío de instrucciones.
- `frontend/src/pages/RecipeDetailPage.css` — clase `.recipe-detail__tabs` (objetivos táctiles ≥48px) y `.recipe-detail__instructions-empty`.
- `frontend/src/pages/RecipeDetailPage.test.tsx` — tests de pestañas (activación por defecto, cambio, persistencia de edición, teclado, roles).
- `frontend/src/components/IngredientEditor.tsx` — sin cambios; se monta dentro del panel de Ingredientes.

## Cómo funciona

- Se usa `Tabs` de antd (dependencia existente) para obtener roles ARIA `tablist`/`tab`/`tabpanel` y navegación por teclado.
- `defaultActiveKey="instructions"` fija la pestaña principal.
- Los paneles **no** se destruyen al cambiar de pestaña (comportamiento por defecto de antd 6, `destroyOnHidden` falsy), de modo que un borrador de edición de ingredientes sobrevive al cambio de pestaña.
- Si `instructions` es `null` o vacío, el panel de Instrucciones muestra «Esta receta no tiene instrucciones.».
- `RecipeDetailContent` sigue montado con `key={id}`, por lo que al navegar a otra receta la pestaña activa se reinicia a Instrucciones.

## Variables / configuración

Ninguna. Sin cambios de backend, API, esquema ni variables de entorno.

## Cómo habilitar / desplegar

Cambio de frontend: se aplica con el build habitual (`npm run build`) y el despliegue estándar del frontend. No requiere migraciones.

## Decisiones

- **D1:** `Tabs` de antd en lugar de un componente propio (a11y y teclado sin coste de mantenimiento).
- **D2:** Instrucciones activa por defecto (contenido principal).
- **D3:** paneles no destructivos para conservar la edición de ingredientes al cambiar de pestaña.
- **D4:** estado vacío explícito cuando no hay instrucciones.
- **D5:** sin reflejar la pestaña en la URL.

## Follow-ups

- Reflejar la pestaña activa en la URL (`?tab=ingredients`) para enlaces compartibles.
- Redundancia menor: la pestaña «Ingredientes» y el `h3` interno de `IngredientEditor` duplican etiqueta para lectores de pantalla (aceptado).
- Analizar mostrar el número de ingredientes en la etiqueta de la pestaña.
===FIN===
