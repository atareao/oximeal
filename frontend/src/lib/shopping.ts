import type { ShoppingItem } from '../api/types';

/** Emoji por pasillo canónico del backend. */
const CATEGORY_EMOJI: Readonly<Record<string, string>> = {
  Verdulería: '🥦',
  Pescadería: '🐟',
  Carnicería: '🥩',
  Refrigerados: '❄️',
  Panadería: '🍞',
  Bebidas: '🥤',
  Despensa: '🥫',
  Otros: '🛒',
};

const FALLBACK_EMOJI = '🛒';

/** Pasillos canónicos en el orden en que se muestran en los selectores. */
export const SHOPPING_CATEGORIES: readonly string[] = Object.keys(CATEGORY_EMOJI);

/** Emoji del encabezado de un pasillo (fallback 🛒). */
export function categoryEmoji(category: string): string {
  return CATEGORY_EMOJI[category] ?? FALLBACK_EMOJI;
}

const amountFormatter = new Intl.NumberFormat('es-ES', {
  maximumFractionDigits: 2,
});

/** Formatea una cantidad sin decimales innecesarios (`2`, `1,5`). */
export function formatAmount(amount: number): string {
  return amountFormatter.format(amount);
}

/**
 * Ordena los ítems de una categoría: pendientes primero, comprados al final.
 * Dentro de cada grupo se conserva el orden por `updated_at` ascendente.
 */
export function sortShoppingItems(items: readonly ShoppingItem[]): ShoppingItem[] {
  return [...items].sort((a, b) => {
    if (a.checked !== b.checked) {
      return a.checked ? 1 : -1;
    }
    const byUpdated = a.updated_at.localeCompare(b.updated_at);
    if (byUpdated !== 0) {
      return byUpdated;
    }
    return a.id.localeCompare(b.id);
  });
}
