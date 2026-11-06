# Proposal

## Why

El planificador ofrece hoy cuatro franjas (desayuno, comida, cena y merienda), pero el uso real del menú se limita a COMIDA y CENA. Mantener desayuno y merienda en la interfaz añade ruido y confusión sin aportar valor. Al mismo tiempo, el backend, la API y la base de datos ya gestionan los cuatro valores y contienen filas históricas, por lo que no conviene alterarlos.

## What Changes

- La vista diaria deja de mostrar las franjas desayuno y merienda: muestra únicamente COMIDA y CENA.
- El selector de "tipo de comida" al planificar deja de ofrecer desayuno y merienda: solo ofrece Comida y Cena.
- El tipo de datos `MealType` se mantiene con los cuatro valores para seguir representando lo que la API puede devolver, incluidas filas existentes.
- No cambia el backend, la API ni la base de datos: `POST /api/meal-plan` sigue aceptando los cuatro valores por compatibilidad.

## Capabilities

### New Capabilities

<!-- Ninguna: el cambio es una modificación de comportamiento existente. -->

### Modified Capabilities

- `weekly-menu`: la vista diaria y el selector de tipo de comida dejan de ofrecer las franjas desayuno y merienda, limitándose a COMIDA y CENA.

## Impact

- **Frontend afectado**: `frontend/src/components/DailyView.tsx` (slots de la vista diaria), `frontend/src/lib/mealPlan.ts` (`MEAL_TYPE_OPTIONS`), `frontend/src/components/AddMealPlanDialog.tsx` (opciones del selector) y `frontend/src/pages/MenuPage.test.tsx` (test de la vista diaria).
- **Sin cambios**: backend, `POST /api/meal-plan`, esquema de base de datos y el tipo `MealType` de `frontend/src/api/types.ts`.
- **Compatibilidad**: las filas existentes con `meal_type` desayuno o merienda siguen siendo válidas; solo dejan de ofrecerse como opciones nuevas.
