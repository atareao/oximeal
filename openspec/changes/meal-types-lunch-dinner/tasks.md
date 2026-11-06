# Tasks

## 1. RED — Tests del nuevo comportamiento

- [ ] 1.1 En `frontend/src/pages/MenuPage.test.tsx`, reemplazar el test `'la vista diaria muestra desayuno, comida, cena y merienda'` por uno que verifique que la vista diaria muestra solo COMIDA y CENA, y comprobar que falla con `npx vitest run` (ejecutado en `frontend/`)
- [ ] 1.2 Añadir un test que verifique que el selector de tipo de comida solo ofrece Comida y Cena (no desayuno ni merienda), y comprobar que falla con `npx vitest run`

## 2. GREEN — Implementación mínima

- [ ] 2.1 Limitar los slots de la vista diaria a COMIDA y CENA en `frontend/src/components/DailyView.tsx` (o apoyarse en el default de `DayCard`), y verificar que pasa el test 1.1 con `npx vitest run`
- [ ] 2.2 Reducir `MEAL_TYPE_OPTIONS` en `frontend/src/lib/mealPlan.ts` a Comida y Cena, y verificar que pasa el test 1.2 con `npx vitest run` y que `npx tsc --noEmit` no reporta errores

## 3. REFACTOR y verificación de calidad

- [ ] 3.1 Mantener la unión `MealType` con los cuatro valores en `frontend/src/api/types.ts` y revisar que no queden referencias muertas a las franjas retiradas en la UI, verificando con `npx tsc --noEmit`
- [ ] 3.2 Ejecutar `npm run lint` en `frontend/` y confirmar que no hay avisos
- [ ] 3.3 Ejecutar la puerta completa `just ci-ui` y confirmar que pasa en verde
