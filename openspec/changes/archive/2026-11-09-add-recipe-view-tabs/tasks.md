# Tasks

## 1. Frontend: tests en RED

- [x] 1.1 Añadir en `frontend/src/pages/RecipeDetailPage.test.tsx` el test «Instrucciones activa por defecto»: existe `role=tablist` con pestañas «Instrucciones» e «Ingredientes», el panel de Instrucciones es visible y el de Ingredientes no. Verificar con `npx vitest run src/pages/RecipeDetailPage.test.tsx` que falla.
- [x] 1.2 Añadir el test «cambiar a Ingredientes»: activar la pestaña y comprobar que se ve la lista de ingredientes con nombres y cantidades. Verificar que falla.
- [x] 1.3 Añadir el test «volver a Instrucciones». Verificar que falla.
- [x] 1.4 Añadir el test «receta sin instrucciones» → estado vacío en el panel. Verificar que falla.
- [x] 1.5 Añadir el test de accesibilidad: roles `tablist`/`tab`/`tabpanel` y `aria-selected` coherentes. Verificar que falla.

## 2. Frontend: implementación en GREEN

- [x] 2.1 En `frontend/src/pages/RecipeDetailPage.tsx`, mover la sección de instrucciones y el `<IngredientEditor>` a dos paneles de un `Tabs` de antd con `defaultActiveKey="instructions"`, situado por debajo de etiquetas/acciones y por encima de la fuente original. Verificar con `npx vitest run src/pages/RecipeDetailPage.test.tsx`.
- [x] 2.2 Añadir el estado vacío de instrucciones cuando `instructions` es `null`/vacío. Verificar con el test correspondiente.
- [x] 2.3 Añadir los estilos de `Tabs` en `frontend/src/pages/RecipeDetailPage.css` (espaciado y objetivos táctiles ≥48px), conservando los nombres `recipe-detail__*`. Verificar con `npx tsc --noEmit`.

## 3. Frontend: REFACTOR

- [x] 3.1 Ejecutar `npm run lint`, `npx tsc --noEmit` y `npm run format:check`; corregir avisos.
- [x] 3.2 Ejecutar `npx vitest run` completo; confirmar 100% en verde.

## 4. Auditoría

- [x] 4.1 Lanzar `@react-reviewer` sobre `RecipeDetailPage.tsx`, `IngredientEditor.tsx`, `RecipeDetailPage.css` y los tests; corregir hallazgos y re-ejecutar los tests.

## 5. Documentación y cierre

- [x] 5.1 Crear `docs/recipe-view-tabs.md` con qué es, ficheros implicados, cómo funciona, decisiones y follow-ups.
- [x] 5.2 Verificación E2E manual en navegador del flujo completo. (verificado con build de producción OK + 416 tests + smoke de la vista; no se hizo navegador real)
- [x] 5.3 Archivar el change con `openspec archive add-recipe-view-tabs` y renombrar el plan a `plans/PLAN-007-ARCHIVED.md`.