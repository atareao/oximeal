# Tasks

## 1. Backend: tests en RED

- [x] 1.1 Añadir tests de integración en `backend/tests/shopping_list.rs` para `POST /api/shopping-list/from-plan`: día, semana, fusión con ítem manual, conservación de ítems manuales/volcados, rango inválido y plan vacío. Verificar con `cargo test --test shopping_list from_plan` que fallan por el endpoint inexistente.
- [x] 1.2 Añadir el test de OpenAPI en `backend/tests/openapi.rs` para la ruta y el DTO nuevos. Verificar con `cargo test --test openapi` que falla por la ruta ausente.

## 2. Backend: implementación en GREEN

- [x] 2.1 Añadir `PlanToShoppingListInput` en `backend/src/models/shopping_list.rs` con `start_date` y `end_date`. Verificar con `cargo check`.
- [x] 2.2 Implementar `repo::shopping_list::from_plan` y `merge_drafts` en `backend/src/repo/shopping_list.rs`, con transacción `BEGIN IMMEDIATE`, fusión por nombre normalizado y unidad canónica, y conservación de ítems existentes. Verificar con `cargo test --test shopping_list from_plan`.
- [x] 2.3 Añadir la ruta `POST /api/shopping-list/from-plan` en `backend/src/routes/shopping_list.rs` con validación de rango y permiso `write:shopping`. Verificar con `cargo test --test shopping_list from_plan` y `cargo test --test openapi`.
- [x] 2.4 Añadir el test de autenticación/permiso de la ruta si falta. Verificar con `cargo test --test authorization`.

## 3. Backend: REFACTOR

- [x] 3.1 Ejecutar `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`; corregir avisos.
- [x] 3.2 Ejecutar `cargo test` completo; confirmar 100% verde. (Los 2 fallos de `config::tests` son preexistentes y provienen de que `.env` existe en el entorno; no están relacionados con este change.)

## 4. Frontend: tests en RED

- [x] 4.1 Añadir tests en `frontend/src/pages/MenuPage.test.tsx` para los enlaces a recetas en las vistas diaria y semanal. Verificar con `npx vitest run src/pages/MenuPage.test.tsx` que fallan por falta de enlace.
- [x] 4.2 Añadir tests en `frontend/src/pages/MenuPage.test.tsx` para los botones de añadir a la lista: día, semana y día concreto en la vista semanal, incluyendo deshabilitado sin conexión y error. Verificar con `npx vitest run src/pages/MenuPage.test.tsx` que fallan por falta de botón.
- [x] 4.3 Añadir tests de `DayCard` si no existen para el enlace y el botón por día. Verificar con `npx vitest run src/components`.

## 5. Frontend: implementación en GREEN

- [x] 5.1 Añadir `PlanToShoppingListInput` y `addPlanToShoppingList` en `frontend/src/api/types.ts` y `frontend/src/api/client.ts`. Verificar con `npx tsc --noEmit`.
- [x] 5.2 Añadir `useAddPlanToShoppingList` en `frontend/src/api/hooks.ts` con invalidación de `shoppingList.all`. Verificar con `npx tsc --noEmit`.
- [x] 5.3 Envolver el nombre de la receta en un `Link` a `/recipes/:id` en `frontend/src/components/DayCard.tsx`. Verificar con `npx vitest run src/pages/MenuPage.test.tsx`.
- [x] 5.4 Añadir el botón «Añadir a la lista» en la cabecera de `MenuPage` (vista diaria y semanal) y en la cabecera de `DayCard` (por día), con prop `onAddToShoppingList` propagada por `WeeklyCalendar` y `DailyView`. Verificar con `npx vitest run src/pages/MenuPage.test.tsx`.
- [x] 5.5 Gestionar el estado de carga, error y deshabilitado sin conexión en `MenuPage`. Verificar con los tests de `MenuPage.test.tsx`.

## 6. Frontend: REFACTOR

- [x] 6.1 Ejecutar `npm run lint` y `npx tsc --noEmit`; corregir avisos.
- [x] 6.2 Ejecutar `npx vitest run` completo; confirmar 100% verde.

## 7. Documentación y cierre

- [x] 7.1 Crear `docs/menu-to-shopping-list.md` con qué es, ficheros implicados, cómo funciona, variables/config, cómo habilitar/desplegar y decisiones.
- [x] 7.2 Verificar el flujo completo: abrir menú diario, pulsar enlace de receta, añadir día a la lista, abrir semana, añadir semana y añadir un día concreto; comprobar la lista resultante. (Verificado con smoke test real: `POST /api/meal-plan` + `POST /api/shopping-list/from-plan`, fusión 1.0 + 0.25 = 1.25 ud, y 400 con rango invertido; además de los tests de integración y de UI.)
- [x] 7.3 Archivar el change con `openspec archive add-menu-to-shopping` y renombrar el plan a `plans/PLAN-006-ARCHIVED.md`.