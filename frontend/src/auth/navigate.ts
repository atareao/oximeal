/**
 * Navegación de navegador completa a una URL (p. ej. el `end_session_endpoint`
 * del IdP en el cierre de sesión iniciado por el RP).
 *
 * Aislada en su propio módulo para poder mockearla en los tests: jsdom no
 * implementa la navegación externa de `window.location.assign`.
 */
export function navigateTo(url: string): void {
  window.location.assign(url);
}
