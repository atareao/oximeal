/**
 * Presentación de recetas: etiquetas de categoría y metadatos.
 *
 * Las categorías son las que valida el backend (`main|salad|side|dessert|breakfast`).
 */

export interface CategoryOption {
  value: string;
  label: string;
}

/** Categorías admitidas por el backend, con su etiqueta en español. */
export const RECIPE_CATEGORIES: readonly CategoryOption[] = [
  { value: 'main', label: 'Principal' },
  { value: 'salad', label: 'Ensalada' },
  { value: 'side', label: 'Guarnición' },
  { value: 'dessert', label: 'Postre' },
  { value: 'breakfast', label: 'Desayuno' },
];

const CATEGORY_LABELS: Readonly<Record<string, string>> = Object.fromEntries(
  RECIPE_CATEGORIES.map((option) => [option.value, option.label]),
);

/** Etiqueta legible de una categoría; devuelve el valor original si no se conoce. */
export function categoryLabel(value: string): string {
  return CATEGORY_LABELS[value] ?? value;
}

/** Resumen textual de tiempo y raciones (omite lo que no esté definido). */
export function formatRecipeMeta(prepTimeMinutes: number | null, servings: number): string {
  const parts: string[] = [];
  if (prepTimeMinutes != null) {
    parts.push(`${prepTimeMinutes} min`);
  }
  parts.push(`${servings} raciones`);
  return parts.join(' · ');
}
