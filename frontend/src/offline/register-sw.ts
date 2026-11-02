/**
 * Registra el Service Worker de activos.
 *
 * Solo en producción: en `pnpm dev` el Service Worker interferiría con el HMR
 * de Vite y con el manifiesto servido por el servidor de desarrollo. Un fallo
 * de registro no debe romper el arranque: la aplicación funciona igual, solo
 * pierde el modo offline.
 */
export function registerServiceWorker(): void {
  if (!import.meta.env.PROD) {
    return;
  }
  if (!('serviceWorker' in navigator)) {
    return;
  }

  const register = () => {
    void navigator.serviceWorker.register('/sw.js', { scope: '/' }).catch((error: unknown) => {
      // Sin Service Worker la aplicación sigue funcionando; solo pierde el modo offline.
      console.warn('No se pudo registrar el Service Worker', error);
    });
  };

  if (document.readyState === 'complete') {
    register();
  } else {
    window.addEventListener('load', register, { once: true });
  }
}
