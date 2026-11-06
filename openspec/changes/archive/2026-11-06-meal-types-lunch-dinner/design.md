# Design

## Context

El planificador de menú (capability `weekly-menu`, solo frontend) permite hoy planificar cuatro franjas: desayuno, comida, cena y merienda. La vista diaria renderiza las cuatro y el diálogo de alta ofrece las cuatro opciones de tipo de comida. El backend expone `POST /api/meal-plan`, que acepta los cuatro valores, y la base de datos almacena un `meal_type` que puede contener cualquiera de ellos, incluidas filas históricas.

El tipo `MealType` de `frontend/src/api/types.ts` es la unión de los cuatro valores y representa el contrato de datos que la API puede devolver. Ver `proposal.md - Why` para la motivación del cambio.

Restricciones relevantes:
- La API y la base de datos no deben cambiar: el endpoint debe seguir aceptando los cuatro valores por compatibilidad y para no romper filas existentes.
- El cambio es exclusivamente de interfaz: qué franjas se muestran y qué opciones se ofrecen al planificar.

## Goals / Non-Goals

**Goals:**
- La vista diaria muestra solo COMIDA y CENA.
- El selector de tipo de comida al planificar ofrece solo Comida y Cena.
- Mantener intacto el contrato de datos: `MealType` sigue aceptando los cuatro valores que la API puede devolver.

**Non-Goals:**
- No modificar el backend, `POST /api/meal-plan`, ni el esquema de base de datos.
- No migrar ni eliminar filas existentes con `meal_type` desayuno o merienda.
- No estrechar la unión `MealType` (ver Decisiones).
- No alterar la vista semanal, que ya se limita a COMIDA y CENA, ni la mensual. Los scenarios añadidos al delta para semanal y mensual son GUARDAS de verificación del comportamiento ya existente o ya limitado, no cambios de comportamiento nuevos.

## Decisions

### Decisión 1: Limitar solo las opciones y los slots mostrados, sin tocar el backend

Se limita la presentación: la vista diaria deja de pasar las franjas desayuno y merienda y el listado `MEAL_TYPE_OPTIONS` se reduce a Comida y Cena.

*Alternativa considerada*: cambiar también el endpoint y la base de datos para rechazar los cuatro valores antiguos. Se descarta porque rompería la compatibilidad con filas existentes y con cualquier cliente que aún envíe esos valores, y porque el alcance acordado es solo UI.

### Decisión 2: Mantener la unión `MealType` con los cuatro valores

`MealType` permanece como `'lunch' | 'dinner' | 'breakfast' | 'snack'` porque describe lo que la API puede devolver, incluidas filas legacy. Solo se recortan las opciones que el usuario puede elegir.

*Alternativa considerada*: estrechar `MealType` a `'lunch' | 'dinner'`. Se descarta porque el tipo modela el contrato de datos de la API, no solo las opciones del formulario; estrecharlo obligaría a conversiones defensivas al leer datos del servidor y podría ocultar errores de tipado con respuestas reales.

### Decisión 3: Reutilizar los slots por defecto de la tarjeta de día

La vista diaria puede apoyarse en el default ya existente de `DayCard` (`['lunch', 'dinner']`) en lugar de pasar explícitamente las cuatro franjas, eliminando la fuente de la discrepancia.

*Alternativa considerada*: mantener el paso explícito de slots y corregir solo la lista. Se acepta como equivalente; la decisión se resuelve durante la implementación priorizando el default para evitar futuras divergencias.

## Risks / Trade-offs

- [Filas existentes con desayuno o merienda dejan de verse en la vista diaria] → Aceptado: el alcance es no mostrarlas; los datos se conservan en backend y siguen siendo válidos. Se documenta para futuras decisiones sobre visualización histórica.
- [Divergencia entre las opciones ofrecidas y el tipo de dato] → Mitigación: las opciones viven en `MEAL_TYPE_OPTIONS` y el tipo `MealType` se mantiene como contrato; un test cubre que el selector solo ofrece COMIDA y CENA.
- [Regresión en la vista diaria al cambiar los slots] → Mitigación: actualizar el test de `MenuPage` para reflejar el nuevo comportamiento y ejecutar `just ci-ui`.

## Migration Plan

1. Ajustar los slots de la vista diaria y las opciones de tipo de comida.
2. Actualizar los tests de la vista diaria y del selector.
3. Verificar con `npx vitest run`, `npx tsc --noEmit` y `npm run lint` en `frontend/`, y con la puerta completa `just ci-ui`.
4. Rollback: revertir el cambio de presentación; no hay migración de datos ni cambios de backend que deshacer.
