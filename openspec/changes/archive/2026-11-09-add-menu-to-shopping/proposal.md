# Proposal

## Why

Las vistas diaria y semanal del menú no permiten abrir la receta planificada ni volcar sus ingredientes a la lista de la compra sin pasar por el detalle de la receta o por la página de la lista. Esto añade pasos innecesarios a una tarea cotidiana y hace fácil perder el hilo entre el menú y la compra.

## What Changes

- Añadir un enlace a la receta en cada entrada planificada de las vistas diaria y semanal.
- Añadir un botón «Añadir a la lista» junto a la fecha en la vista diaria, que añade los ingredientes del día visible.
- Añadir un botón «Añadir a la lista» junto a la fecha en la vista semanal, que añade los ingredientes de la semana completa.
- Añadir un botón «Añadir a la lista» en cada tarjeta de día de la vista semanal, que añade solo los ingredientes de ese día.
- Añadir el endpoint `POST /api/shopping-list/from-plan` que fusiona los ingredientes del plan con la lista existente, conservando los ítems manuales y los ya volcados.
- La operación se deshabilita sin conexión y refleja estados de carga y error.

## Capabilities

### New Capabilities

<!-- Ninguna: se modifican capacidades existentes. -->

### Modified Capabilities

- `weekly-menu`: se añaden enlaces a la receta y acciones de añadir a la lista desde la vista diaria y semanal.
- `shopping-list`: se añade el volcado aditivo desde el plan, con fusión de ingredientes y conservación de los ítems existentes.

## Impact

- Backend: nuevo endpoint `POST /api/shopping-list/from-plan`, lógica de repo en `backend/src/repo/shopping_list.rs`, DTO en `backend/src/models/shopping_list.rs`, ruta en `backend/src/routes/shopping_list.rs`.
- Frontend: `MenuPage.tsx`, `DayCard.tsx`, `WeeklyCalendar.tsx`, `DailyView.tsx`, `api/client.ts`, `api/hooks.ts`, `api/types.ts`.
- Tests: integración backend en `backend/tests/shopping_list.rs`; frontend en `MenuPage.test.tsx` y tests de componentes.
- Documentación: `docs/menu-to-shopping-list.md` y actualización de `openspec/specs/shopping-list/spec.md` y `openspec/specs/weekly-menu/spec.md` al archivar.
- Sin cambios de esquema ni migraciones.