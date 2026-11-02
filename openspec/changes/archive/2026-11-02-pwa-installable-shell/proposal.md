# Change: Shell PWA instalable

## Why
La Fase 5 arranca con el shell instalable. Hoy `index.html` ya declara metadatos de app móvil y la CSP contempla `manifest-src 'self'` y `worker-src 'self' blob:`, pero no existen manifiesto, iconos ni Service Worker: la aplicación no es instalable ni arranca sin conexión.

## What Changes
- Añadir `frontend/public/manifest.webmanifest` (`name`, `short_name`, `start_url`, `scope`, `display: standalone`, colores, `lang: es` e iconos 192/512/maskable).
- Añadir los iconos en `frontend/public/icons/` (192, 512, 512 maskable y apple-touch 180) y `favicon.svg`.
- Añadir `frontend/public/sw.js`: precache del shell, *cache-first* para `assets/*`, *network-first* para navegaciones con fallback al `index.html` cacheado y limpieza de cachés antiguas en `activate`. No intercepta `/api/*`.
- Registrar el Service Worker solo en producción (`frontend/src/offline/register-sw.ts`, invocado desde `main.tsx`).
- `frontend/index.html`: enlazar manifiesto, favicon y `apple-touch-icon`.
- `backend/src/static_files/embedded.rs`: servir `.webmanifest` como `application/manifest+json` y garantizar `Cache-Control: no-cache` en `sw.js`.

## Impact
- Capabilities: nueva `pwa-shell`; delta en `frontend-serving`.
- Ficheros: `frontend/public/**`, `frontend/index.html`, `frontend/src/main.tsx`, `frontend/src/offline/register-sw.ts`, `backend/src/static_files/embedded.rs`, `PLAN.md`, `CHANGELOG.md`.
- Sin dependencias nuevas. El manifiesto, los iconos y el Service Worker viven en `public/`, fuera del grafo del bundle, por lo que no afectan al presupuesto de 500 kB.

Cierra el issue #29.
