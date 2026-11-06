# Tasks

## 1. RED — Tests del nuevo comportamiento

- [x] 1.1 En `frontend/src/pages/MenuPage.test.tsx`, reemplazar el test `'la vista diaria muestra desayuno, comida, cena y merienda'` por uno que verifique que la vista diaria muestra solo COMIDA y CENA, y comprobar que falla con `npx vitest run` (ejecutado en `frontend/`)
- [x] 1.2 Añadir un test que verifique que el selector de tipo de comida solo ofrece Comida y Cena (no desayuno ni merienda), y comprobar que falla con `npx vitest run`

## 2. GREEN — Implementación mínima

- [x] 2.1 Limitar los slots de la vista diaria a COMIDA y CENA en `frontend/src/components/DailyView.tsx` (o apoyarse en el default de `DayCard`), y verificar que pasa el test 1.1 con `npx vitest run`
- [x] 2.2 Reducir `MEAL_TYPE_OPTIONS` en `frontend/src/lib/mealPlan.ts` a Comida y Cena, y verificar que pasa el test 1.2 con `npx vitest run` y que `npx tsc --noEmit` no reporta errores

## 3. REFACTOR y verificación de calidad

- [x] 3.1 Mantener la unión `MealType` con los cuatro valores en `frontend/src/api/types.ts` y revisar que no queden referencias muertas a las franjas retiradas en la UI, verificando con `npx tsc --noEmit`
- [x] 3.2 Ejecutar `npm run lint` en `frontend/` y confirmar que no hay avisos
- [x] 3.3 Ejecutar la puerta completa `just ci-ui` y confirmar que pasa en verde
- [x] 3.4 (Trabajo extra post-auditoría) Cubrir con test el escenario de guarda del resumen mensual (solo COMIDA y CENA, excluyendo franjas legacy) en `frontend/src/components/MonthlyCalendar.test.tsx` y corregir el filtro en `frontend/src/components/MonthlyCalendar.tsx`. Verificación: `npx vitest run`
