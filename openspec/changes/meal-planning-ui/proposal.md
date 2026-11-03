# Change: planificación de comidas en la UI (nativa, en convivencia con Valet)

## Why

La planificación de comidas nace como **solo lectura** en la interfaz. El
backend ya expone el ciclo de escritura (`POST` y `DELETE` sobre
`/api/meal-plan`, permisos `read:plan`/`write:plan`) y `MenuPage` pinta el plan
semanal con `WeeklyCalendar` y `DayCard`, pero el usuario **no puede crear,
editar ni quitar** una entrada desde la app: el estado vacío se limita a remitir
al asistente Valet. Para planificar hace falta salir de OxiMeal, lo que rompe el
principio de que el usuario gestiona sus propios datos desde la interfaz.

Además, no existe forma de corregir las raciones de una entrada ya creada: no
hay `PATCH`, así que la única vía es borrar y volver a crear, perdiendo el
`id`.

Este change (issue **#54**, P1 de la **Fase 7 — producto**, ADR **ADR-34**)
convierte el menú semanal en una vista de planificación editable: alta de una
receta en un slot con selector que reutiliza el recetario, baja con confirmación
y edición de raciones, manteniendo Valet disponible como vía alternativa
(convivencia, no sustitución).

## What Changes

- **Decisión de backend — `PATCH /api/meal-plan/{id}` SÍ se añade.** `POST` y
  `DELETE` cubren alta y baja, pero no permiten *corregir* una entrada sin
  recrearla. Se añade `PATCH /api/meal-plan/{id}` (permiso `write:plan`) para
  editar las **raciones** de una entrada existente, devolviendo `200` con la
  `MealPlanEntryView` actualizada, `400` si `servings < 1` y `404` si la entrada
  no existe. El reemplazo de receta, fecha, `meal_type` o `role` queda **fuera de
  alcance**: se resuelve quitando y volviendo a añadir, para mantener el
  contrato mínimo y sin ambigüedad de conflicto.
- **Endurecer `POST /api/meal-plan`**: documentar y cubrir con tests la
  validación ya implementada —`meal_type ∈ {lunch,dinner,breakfast,snack}`,
  `role ∈ {main,salad,side}`, `date` ISO válida, `recipe_id` no vacío y
  **existente**, y `servings >= 1` cuando se indica (por defecto `2`)— más el
  permiso `write:plan` (401/403).
- **UI de planificación en `MenuPage`/`DayCard`**: cada slot COMIDA/CENA ofrece
  una acción «Añadir» que abre un selector de receta con búsqueda (reutilizando
  `useRecipes` y el patrón de `RecipeCard`), más la elección de **tipo de comida**
  (si procede desde la vista diaria), **rol** (`main|salad|side`) y **raciones**;
  cada entrada planificada ofrece «Quitar» con **confirmación** y «Editar
  raciones». El estado vacío sigue ofreciendo Valet y la planificación nativa pasa
  a ser el camino principal sin retirar el asistente.
- **Hooks de mutación** en `frontend/src/api/hooks.ts`:
  `useAddMealPlanEntry`, `useRemoveMealPlanEntry` y `useUpdateMealPlanEntry`, con
  funciones de cliente en `client.ts` (`createMealPlanEntry`, `deleteMealPlanEntry`,
  `patchMealPlanEntry`) y tipos espejo en `types.ts`. Invalidan
  `queryKeys.mealPlan.all` y, cuando la planificación puede alterar la compra,
  `queryKeys.shoppingList.all`.
- **Offline: la planificación requiere conexión.** A diferencia de las escrituras
  de la lista de la compra, las mutaciones del plan **no se encolan** en
  IndexedDB. Motivo: la existencia de la receta se valida en el servidor y el alta
  introduce un `id` que la cola tendría que reconciliar sin un caso de uso claro;
  encolarlas prometería una persistencia que hoy no existe. Sin conexión la UI
  **deshabilita** los controles de planificación y muestra un aviso degradado,
  coherente con `offline-data`/`offline-writes`.
- **Accesibilidad y mobile-first**: objetivos táctiles ≥48px, gestión de foco al
  abrir/cerrar el selector y la confirmación, etiquetas/roles `aria` en acciones
  y diálogos, y feedback de carga (`pending` del selector/acción) y de error.

## Impact

- Capabilities (deltas):
  - `meal-plan`: `MODIFIED` asignación de slot y eliminación (permisos y
    validación), `ADDED` edición de raciones (`PATCH`).
  - `weekly-menu`: `MODIFIED` vista semanal (de solo lectura a editable),
    `ADDED` alta en slot, baja con confirmación, validación/feedback,
    accesibilidad táctil y convivencia con Valet.
  - `ui-shell`: `MODIFIED` objetivos táctiles (incluyen los controles de
    planificación del menú).
  - `offline-writes`: `ADDED` planificación de comidas requiere conexión.
- Backend: `backend/src/routes/meal_plan.rs` (handler `PATCH` y registro en
  `write_router` + tabla de permisos en `backend/src/routes/mod.rs`),
  `backend/src/models/meal_plan.rs` (DTO de patch), `backend/src/repo/meal_plan.rs`
  (update de raciones) y tests de integración en `backend/tests/`.
- Frontend: `frontend/src/pages/MenuPage.tsx`, `frontend/src/components/DayCard.tsx`
  y/o `WeeklyCalendar.tsx`, nuevos componentes de selector y confirmación,
  `frontend/src/api/{hooks.ts,client.ts,types.ts}` y tests de componente/hook.
- Sin dependencias nuevas.
- Trazabilidad: issue **#54**; ADR **ADR-34**.
