/**
 * Presentación y opciones de la planificación de comidas.
 *
 * Los valores espejan las uniones `MealType`/`MealRole` del backend.
 */

import type { MealRole, MealType } from '../api/types';

/**
 * Franjas planificables (COMIDA y CENA). Fuente única de verdad para la UI:
 * el resumen del calendario y los slots por defecto del día.
 */
export const PLANNED_MEAL_TYPES: readonly MealType[] = ['lunch', 'dinner'];

/** Etiquetas de las franjas de comida, en mayúsculas como en el calendario. */
export const MEAL_TYPE_LABELS: Readonly<Record<MealType, string>> = {
  lunch: 'COMIDA',
  dinner: 'CENA',
  // `breakfast`/`snack` se conservan solo para cumplir el contrato
  // `Record<MealType, string>`; la UI no los ofrece.
  breakfast: 'DESAYUNO',
  snack: 'MERIENDA',
};

/** Franjas ofrecidas al planificar fuera de un slot (estado vacío). */
export const MEAL_TYPE_OPTIONS: readonly { value: MealType; label: string }[] = [
  { value: 'lunch', label: 'Comida' },
  { value: 'dinner', label: 'Cena' },
];

/** Roles del plato en una comida, con su etiqueta en español. */
export const MEAL_ROLE_OPTIONS: readonly { value: MealRole; label: string }[] = [
  { value: 'main', label: 'Principal' },
  { value: 'salad', label: 'Ensalada' },
  { value: 'side', label: 'Guarnición' },
];

/** Raciones por defecto al planificar una receta (coincide con el backend). */
export const DEFAULT_SERVINGS = 2;

/** Etiqueta legible de una franja; devuelve el valor original si se desconoce. */
export function mealTypeLabel(value: string): string {
  return MEAL_TYPE_LABELS[value as MealType] ?? value;
}
