# Design

## Context

La lista de la compra es una única lista global. El endpoint `POST /api/shopping-list/consolidate` reemplaza las líneas derivadas del plan (`source_plan_id` no nulo) y conserva los ítems manuales y los volcados de receta. El endpoint `POST /api/shopping-list/from-recipe` registra una receta en `shopping_recipe_sources` y reconstruye todas las líneas derivadas de recetas. Las vistas diaria y semanal del menú solo muestran las entradas y no ofrecen enlaces ni volcado a la lista.

## Goals / Non-Goals

**Goals:**

- Añadir un enlace a la receta en cada entrada de las vistas diaria y semanal.
- Añadir un botón de volcado a la lista en la vista diaria, en la cabecera de la vista semanal y en cada tarjeta de día.
- Añadir un endpoint aditivo que fusione los ingredientes del plan con la lista existente sin borrar ítems.
- Mantener la lista como una única lista global y conservar los ítems manuales, los volcados de receta y los estados `checked`/`client_id`.

**Non-Goals:**

- Cambiar el esquema de la base de datos o crear migraciones.
- Cambiar el comportamiento de `consolidate` ni de `from-recipe`.
- Añadir una lista por día o por semana.
- Sincronizar el volcado offline.

## Decisions

### Endpoint nuevo y aditivo

Se añade `POST /api/shopping-list/from-plan` con `{ start_date, end_date }`. Se prefiere a reutilizar `consolidate` porque `consolid,ate` reemplaza las líneas del plan y el requisito es conservar la lista existente. La operación se ejecuta en una transacción `BEGIN IMMEDIATE` por el patrón leer-luego-escribir en WAL.

### Fusión por nombre y unidad canónica

Se leen los borradores del plan con `collect_drafts` (ya escala por raciones y consolida con el motor de dominio). Para cada borrador se busca un ítem existente con la misma clave `(nombre normalizado, familia, unidad canónica)`. Si existe, se suma la cantidad en la unidad del ítem existente; si no, se inserta un ítem nuevo con `source_plan_id = NULL` y `source_recipe_id = NULL`. Así los ítems existentes conservan su `id`, `checked` y `client_id`, y los nuevos se comportan como ítems manuales (no los borra `consolidate`).

Alternativa considerada: usar `source_plan_id = 'plan:<date>'` para poder reemplazar el volcado de un día. Se descarta porque obliga a borrar el ítem existente y perder su `checked`/`client_id`, y porque el usuario eligió una semántica de suma.

### Repetición de la llamada

Cada llamada vuelve a sumar los ingredientes del rango. Es coherente con la semántica de «añadir» elegida y permite añadir el mismo menú dos veces si se cocina dos veces. La UI deshabilita el botón mientras la petición está en curso para evitar dobles clics accidentales.

### Frontend

Se añade `addPlanToShoppingList` en `api/client.ts`, `useAddPlanToShoppingList` en `api/hooks.ts` y el tipo `PlanToShoppingListInput`. La mutación usa `networkMode: 'online'` (no se encola) y, al terminar, invalida `queryKeys.shoppingList.all`. `MenuPage` centraliza el handler, muestra el mensaje de éxito y el error. `DayCard` recibe `onAddToShoppingList` y envuelve el nombre de la receta en un `Link` a `/recipes/:id`. `WeeklyCalendar` y `DailyView` propagan el handler.

## Risks / Trade-offs

- [Sumar dos veces el mismo menú] → La UI deshabilita el botón durante la petición y el mensaje de éxito deja claro que se ha añadido.
- [Conversión de unidades al fusionar] → Se usa la misma canonización del motor de dominio; si el ítem existente usa otra unidad de la misma familia, se convierte la suma a la unidad del ítem.
- [Ítems añadidos y luego «Consolidar»] → Los ítems añadidos son manuales y `consolidate` los conserva; el usuario puede revisar la lista resultante.
- [Sin conexión] → El botón se deshabilita y la acción no se encola.

## Migration Plan

Sin cambios de esquema ni migraciones. El endpoint es nuevo y aditivo. Para revertir, se elimina la ruta y el botón; los ítems añadidos quedan como ítems manuales y se pueden borrar desde la lista.

## Open Questions

Ninguna.