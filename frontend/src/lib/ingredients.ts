/**
 * Utilidades de ingredientes para la edición en el detalle.
 */

/**
 * Convierte el texto de un campo de cantidad a número.
 *
 * Acepta coma decimal (formato español) además del punto. Devuelve `null`
 * cuando el texto está vacío o no representa un número finito, de modo que la
 * validación pueda distinguir «no es un número» de cualquier otro valor.
 */
export function parseAmount(value: string): number | null {
  const normalized = value.trim().replace(',', '.');
  if (normalized === '') {
    return null;
  }
  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}
