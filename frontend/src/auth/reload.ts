/**
 * Recarga completa de la aplicación.
 *
 * Aislado en su propio módulo para poder mockearlo en los tests: jsdom no
 * implementa `window.location.reload`.
 */
export function reloadApp(): void {
  window.location.reload();
}
