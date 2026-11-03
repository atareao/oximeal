# Tasks — Planificación de comidas en la UI (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Leer `backend/src/routes/meal_plan.rs`, `backend/src/models/meal_plan.rs`, `backend/src/repo/meal_plan.rs` y la tabla de permisos en `backend/src/routes/mod.rs`
- [ ] 1.2 Leer `frontend/src/pages/MenuPage.tsx`, `frontend/src/components/DayCard.tsx`, `frontend/src/components/WeeklyCalendar.tsx` y `frontend/src/api/{hooks.ts,client.ts,types.ts}`
- [ ] 1.3 Confirmar el patrón de degradado offline y de encolado en `offline-data`/`offline-writes`
- [ ] 1.4 Verificar la decisión de backend (`PATCH` para raciones) y el contrato de `POST` en `backend/src/routes/validation.rs`

## 2. RED
- [ ] 2.1 Test integración backend: `POST /api/meal-plan` válido devuelve `201` con la entrada y aparece en el rango
- [ ] 2.2 Test integración backend: `POST` con `recipe_id` inexistente devuelve `400` y no persiste
- [ ] 2.3 Test integración backend: `POST` sin `servings` usa por defecto `2`; con `servings < 1` devuelve `400`
- [ ] 2.4 Test integración backend: `POST`/`PATCH`/`DELETE` sin permiso devuelven `401`/`403`
- [ ] 2.5 Test integración backend: `PATCH /api/meal-plan/{id}` con `servings` válido devuelve `200` y conserva receta y slot
- [ ] 2.6 Test integración backend: `PATCH` con `servings < 1` devuelve `400` y no modifica
- [ ] 2.7 Test integración backend: `PATCH`/`DELETE` con id inexistente devuelven `404`
- [ ] 2.8 Test frontend (vitest + Testing Library): añadir receta a un slot con búsqueda, rol y raciones
- [ ] 2.9 Test frontend: el tipo de comida del slot se usa sin volver a elegirlo
- [ ] 2.10 Test frontend: quitar una entrada exige confirmación y actualiza el calendario; cancelar no borra
- [ ] 2.11 Test frontend: editar raciones persiste con `PATCH` y conserva la entrada
- [ ] 2.12 Test frontend: validación en cliente (sin receta, raciones < 1) y feedback de carga/error
- [ ] 2.13 Test frontend: accesibilidad (≥48px, foco en selector/diálogo, nombres accesibles)
- [ ] 2.14 Test frontend: sin conexión los controles se deshabilitan con aviso y no se encola
- [ ] 2.15 Test frontend: el estado vacío y la vista mantienen la acción de Valet (convivencia)
- [ ] 2.16 Test de hooks: invalidación de `meal-plan` (y `shopping-list` cuando proceda) tras alta, baja y edición
- [ ] 2.17 Ejecutar `cargo test` y `npx vitest run` y confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Backend: `PATCH` en `backend/src/routes/meal_plan.rs`, DTO en `models/meal_plan.rs` y update en `repo/meal_plan.rs`
- [ ] 3.2 Backend: registrar el handler en `write_router` y documentar el permiso `write:plan` en la tabla de `routes/mod.rs`
- [ ] 3.3 Frontend: funciones `createMealPlanEntry`/`deleteMealPlanEntry`/`patchMealPlanEntry` y tipos espejo en `client.ts`/`types.ts`
- [ ] 3.4 Frontend: hooks `useAddMealPlanEntry`, `useRemoveMealPlanEntry` y `useUpdateMealPlanEntry` con invalidación de queries
- [ ] 3.5 Frontend: selector de recetas con búsqueda, rol y raciones; acciones de añadir/quitar/editar en el día
- [ ] 3.6 Frontend: confirmación de baja y feedback de carga/error; deshabilitar controles sin conexión con aviso
- [ ] 3.7 Frontend: accesibilidad (≥48px, foco, `aria`) y conservación de la vía Valet
- [ ] 3.8 `cargo test` y `npx vitest run` en verde; `cargo check` y `npx tsc --noEmit` sin errores

## 4. REFACTOR
- [ ] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings`
- [ ] 4.2 `npm run lint` y `pnpm format:check` en el frontend
- [ ] 4.3 Re-ejecutar `cargo test` y `npx vitest run` para descartar regresiones

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` y `CHANGELOG.md`
- [ ] 5.3 Archivar el change con `openspec archive meal-planning-ui`
