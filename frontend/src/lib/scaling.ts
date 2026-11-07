/**
 * Utilidades de escalado de cantidades para el detalle de receta.
 *
 * El cálculo de la lista de la compra vive en el backend
 * (`repo/shopping_list.rs`); aquí solo se replica el redondeo de presentación
 * para no mostrar cantidades con precisión absurda al cambiar de raciones.
 */

/**
 * Escala una cantidad por un factor y la redondea según su magnitud:
 * - resultado >= 10 → entero
 * - resultado >= 1 → un decimal
 * - resultado < 1 → dos decimales
 */
export function scaleAmount(amount: number, factor: number): number {
  if (
    !Number.isFinite(amount) ||
    !Number.isFinite(factor) ||
    amount < 0 ||
    factor < 0
  ) {
    return 0;
  }
  const result = amount * factor;
  if (result >= 10) {
    return Math.round(result);
  }
  if (result >= 1) {
    return Math.round(result * 10) / 10;
  }
  return Math.round(result * 100) / 100;
}

/**
 * Indica si una receta tiene cantidades reales.
 *
 * La importación guarda los ingredientes sin parsear (`amount = 1`,
 * `unit = "ud"`). Si **todos** los ingredientes tienen esa forma, no tiene
 * sentido escalar (produciría cantidades como `0,25 ud`). Un array vacío se
 * considera con cantidades reales (no hay nada que delate una importación).
 */
export function hasRealQuantities(ingredients: { amount: number; unit: string }[]): boolean {
  if (ingredients.length === 0) {
    return true;
  }
  return !ingredients.every((ingredient) => ingredient.amount === 1 && ingredient.unit === 'ud');
}

const scaledAmountFormatter = new Intl.NumberFormat('es-ES', {
  maximumFractionDigits: 2,
});

/** Formatea una cantidad escalada con coma decimal y sin decimales sobrantes. */
export function formatScaledAmount(value: number): string {
  if (!Number.isFinite(value)) {
    return '0';
  }
  return scaledAmountFormatter.format(value);
}
