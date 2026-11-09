# PLAN-006 — Enlaces a recetas y añadir a la lista desde el menú diario y semanal

- **Issue:** #123
- **Change OpenSpec:** `add-menu-to-shopping`
- **Estado:** Pendiente de aprobación e implementación
- **Rama:** `feature/menu-to-shopping-list` desde `development`

## 1. Objetivo

Facilitar el acceso a la receta desde las vistas diaria y semanal del menú y permitir volcar sus ingredientes a la lista de la compra sin salir del menú. El volcado debe ser aditivo y fusionarse con la lista existente.

## 2. Alcance

### Backend

- Nuevo endpoint `POST /api/shopping-list/from-plan` con `{ start_date, end_date }`.
- Validación: ambas fechas obligatorias, `YYYY-MM-DD`, `start_date <= end_date`.
- Permiso `write:shopping`.
- Escalado por raciones de la entrada y de la receta.
- Consolidación con el motor de dominio.
- Fusión con los ítems existentes por nombre normalizado y unidad canónica.
- Conservación de ítems manuales, volcados de receta y estados `checked`/`client_id`.
- Sin cambios de esquema ni migraciones.

### Frontend

- Enlace a `/recipes/:id` en cada entrada de las vistas diaria y semanal.
- Botón «Añadir a la lista» en la vista diaria (día visible).
- Botón «Añadir a la lista» en la cabecera de la vista semanal (semana completa).
- Botón «Añadir a la lista» en cada tarjeta de día de la vista semanal (solo ese día).
- Estados de carga, error y deshabilitado sin conexión.
- Invalidación de `shoppingList.all` tras el éxito.

## 3. Fases

1. **Spec aprobada:** `proposal.md`, `specs/weekly-menu/spec.md`, `specs/shopping-list/spec.md`, `design.md` y `tasks.md`.
2. **Backend TDD:** tests en RED, implementación en GREEN, `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`.
3. **Frontend TDD:** tests en RED, implementación en GREEN, `npm run lint` y `npx tsc --noEmit`.
4. **Integración:** flujo completo en navegador y tests completos.
5. **Cierre:** `docs/menu-to-shopping-list.md`, archivar el change y renombrar el plan a `plans/PLAN-006-ARCHIVED.md`.

## 4. Criterios de aceptación

- [ ] Cada receta de las vistas diaria y semanal enlaza a su detalle.
- [ ] La vista diaria tiene un botón «Añadir a la lista» junto a la fecha que añade los ingredientes del día.
- [ ] La vista semanal tiene un botón «Añadir a la lista» junto a la fecha que añade los ingredientes de la semana.
- [ ] Cada día de la vista semanal tiene su propio botón «Añadir a la lista» para añadir solo ese día.
- [ ] La operación añade/fusiona los ingredientes con la lista actual sin borrar los ítems existentes.
- [ ] La UI refleja el estado de carga/error y deshabilita la acción sin conexión.
- [ ] Tests backend y frontend cubren los escenarios.
- [ ] `cargo test`, `cargo clippy --all-targets -- -D warnings`, `npx vitest run`, `npx tsc --noEmit` y `npm run lint` en verde.

## 5. Verificación

- Backend: `cargo test --test shopping_list from_plan`, `cargo test --test openapi`, `cargo test --test authorization`, `cargo test`.
- Frontend: `npx vitest run src/pages/MenuPage.test.tsx`, `npx vitest run`, `npx tsc --noEmit`, `npm run lint`.
- Manual: abrir `/menu?view=day`, pulsar el enlace de receta, añadir el día; abrir `/menu?view=week`, añadir la semana y añadir un día concreto; revisar `/shopping`.

## 6. Riesgos

- Repetir la llamada suma de nuevo; la UI deshabilita el botón durante la petición.
- La fusión de unidades usa la canonización del motor de dominio; revisar los tests de unidades.
- Los ítems añadidos son manuales y `consolidate` los conserva; no se borran al consolidar.