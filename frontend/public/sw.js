/* Service Worker de activos de OxiMeal.
 *
 * No intercepta `/api/*` ni `/auth/*`: la caché de datos vive en IndexedDB
 * (capa de datos), no en el HTTP cache del Service Worker. Así no se cachean
 * respuestas autenticadas ni se filtran datos entre sesiones.
 *
 * `__CACHE_VERSION__` y `PRECACHE_ASSETS` los inyecta `scripts/build-sw.mjs`
 * a partir del manifest de Vite: la versión es el hash del listado de assets,
 * de modo que un build nuevo invalida las cachés anteriores sin bumps manuales.
 */
const CACHE_VERSION = '__CACHE_VERSION__';
const CACHE_PREFIX = 'oximeal-';
const SHELL_CACHE = `${CACHE_PREFIX}shell-${CACHE_VERSION}`;
const ASSET_CACHE = `${CACHE_PREFIX}assets-${CACHE_VERSION}`;

const PRECACHE_ASSETS = [];

const SHELL_URLS = [
  '/index.html',
  '/manifest.webmanifest',
  '/favicon.svg',
  '/icons/icon-192.png',
  '/icons/icon-512.png',
  '/icons/icon-512-maskable.png',
  '/icons/apple-touch-icon-180.png',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    Promise.all([
      caches.open(SHELL_CACHE).then((cache) => cache.addAll(SHELL_URLS)),
      caches.open(ASSET_CACHE).then((cache) => cache.addAll(PRECACHE_ASSETS)),
    ]),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter(
              (key) => key.startsWith(CACHE_PREFIX) && key !== SHELL_CACHE && key !== ASSET_CACHE,
            )
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') {
    return;
  }

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) {
    return;
  }
  if (isApiOrAuth(url.pathname)) {
    return;
  }

  if (request.mode === 'navigate') {
    event.respondWith(networkFirstNavigation(request));
    return;
  }

  if (url.pathname.startsWith('/assets/')) {
    event.respondWith(cacheFirst(request, ASSET_CACHE));
    return;
  }

  if (isShellAsset(url.pathname)) {
    event.respondWith(staleWhileRevalidate(event, request, SHELL_CACHE));
  }
});

function isApiOrAuth(pathname) {
  return (
    pathname === '/api' ||
    pathname.startsWith('/api/') ||
    pathname === '/auth' ||
    pathname.startsWith('/auth/')
  );
}

function isShellAsset(pathname) {
  return (
    pathname === '/manifest.webmanifest' ||
    pathname === '/favicon.svg' ||
    pathname.startsWith('/icons/')
  );
}

async function networkFirstNavigation(request) {
  try {
    const response = await fetch(request);
    if (response.ok && !response.redirected) {
      const cache = await caches.open(SHELL_CACHE);
      await cache.put('/index.html', response.clone());
    }
    return response;
  } catch {
    const cache = await caches.open(SHELL_CACHE);
    return (await cache.match('/index.html')) ?? Response.error();
  }
}

async function cacheFirst(request, cacheName) {
  const cache = await caches.open(cacheName);
  const cached = await cache.match(request);
  if (cached) {
    return cached;
  }
  try {
    const response = await fetch(request);
    if (response.ok) {
      await cache.put(request, response.clone());
    }
    return response;
  } catch {
    return Response.error();
  }
}

function staleWhileRevalidate(event, request, cacheName) {
  const network = (async () => {
    const cache = await caches.open(cacheName);
    const response = await fetch(request);
    if (response.ok) {
      await cache.put(request, response.clone());
    }
    return response;
  })().catch(() => undefined);

  // `waitUntil` se invoca de forma síncrona, dentro del callback del evento.
  event.waitUntil(network);

  return (async () => {
    const cache = await caches.open(cacheName);
    const cached = await cache.match(request);
    return cached ?? (await network) ?? Response.error();
  })();
}
