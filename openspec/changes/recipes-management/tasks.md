# Tasks — Gestión de recetas (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Leer `backend/src/routes/recipes.rs`, `backend/src/repo/recipes.rs`, `backend/src/models/recipe.rs` y la tabla de permisos en `backend/src/routes/mod.rs`
- [ ] 1.2 Leer `frontend/src/api/hooks.ts`, `frontend/src/api/client.ts`, `frontend/src/App.tsx`, `AppLayout.tsx` y `BottomNav.tsx`
- [ ] 1.3 Confirmar el patrón de degradado offline en las capacidades `offline-data`/`offline-writes`

## 2. RED
- [ ] 2.1 Test integración backend: `PUT /api/recipes/{id}` válido devuelve 200 con la receta actualizada (y FTS refleja el cambio)
- [ ] 2.2 Test integración backend: `PUT` con id inexistente devuelve 404
- [ ] 2.3 Test integración backend: `PUT` con payload inválido devuelve 400 y no modifica
- [ ] 2.4 Test integración backend: `PUT`/`DELETE` sin permiso devuelven 401/403
- [ ] 2.5 Test integración backend: `DELETE /api/recipes/{id}` válido devuelve 204 y borra receta e ingredientes
- [ ] 2.6 Test integración backend: `DELETE` con id inexistente devuelve 404
- [ ] 2.7 Test integración backend: tras `DELETE` la receta deja de aparecer en `GET /api/recipes`
- [ ] 2.8 Test frontend (vitest + Testing Library): `Recipes` lista, busca y pagina
- [ ] 2.9 Test frontend: detalle de receta y estado no encontrado
- [ ] 2.10 Test frontend: alta y edición validan y guardan
- [ ] 2.11 Test frontend: borrado exige confirmación y actualiza la vista
- [ ] 2.12 Test frontend: accesibilidad (objetivo ≥48px, etiquetas/roles) y degradado offline
- [ ] 2.13 Test frontend: `AppLayout` y `BottomNav` muestran Recetas y el estado activo en `/recipes`
- [ ] 2.14 Ejecutar `cargo test` y `npx vitest run` y confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Backend: `PUT` en `backend/src/routes/recipes.rs` + repo, reutilizando validaciones y `BEGIN IMMEDIATE`
- [ ] 3.2 Backend: `DELETE` en `backend/src/routes/recipes.rs` + repo (borrado atómico de receta e ingredientes)
- [ ] 3.3 Registrar los handlers en `read_router`/`write_router` y en la tabla de permisos
- [ ] 3.4 Frontend: página `Recipes` (listado, búsqueda, paginación, detalle, alta, edición, borrado)
- [ ] 3.5 Frontend: rutas en `App.tsx` y entradas en `AppLayout.tsx` y `BottomNav.tsx` (🍳 Recetas)
- [ ] 3.6 `cargo test` y `npx vitest run` en verde; `cargo check` y `npx tsc --noEmit` sin errores

## 4. REFACTOR
- [ ] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings`
- [ ] 4.2 `npm run lint` y `pnpm format:check` en el frontend
- [ ] 4.3 Re-ejecutar `cargo test` y `npx vitest run` para descartar regresiones

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` y `CHANGELOG.md`
- [ ] 5.3 Archivar el change con `openspec archive recipes-management`
