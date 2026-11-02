/**
 * Construye la URL del endpoint del backend `GET /auth/login` conservando el
 * destino interno al que volver tras autenticarse.
 */
export function loginUrl(returnTo?: string): string {
  if (!returnTo) {
    return '/auth/login';
  }
  return `/auth/login?return_to=${encodeURIComponent(returnTo)}`;
}

/**
 * Navega a la ruta de login del backend. Es una ruta server-side (redirección
 * al IdP), por lo que se usa `window.location.assign` en lugar del router.
 */
export function redirectToLogin(returnTo?: string): void {
  window.location.assign(loginUrl(returnTo));
}
