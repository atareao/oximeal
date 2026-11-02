import { readFileSync } from 'node:fs';
import { describe, expect, it, vi } from 'vitest';

// La base se guarda en una variable para que Vite no reescriba `new URL(...)`
// como un activo del bundle: aquí necesitamos la ruta real en disco.
const baseUrl = import.meta.url;
const source = readFileSync(new URL('../../public/sw.js', baseUrl), 'utf8')
  .replace("'__CACHE_VERSION__'", "'test'")
  .replace('const PRECACHE_ASSETS = [];', "const PRECACHE_ASSETS = ['/assets/app.js'];");

const SHELL_CACHE = 'oximeal-shell-test';
const ASSET_CACHE = 'oximeal-assets-test';
const ASSET_URL = 'https://oximeal.test/assets/app.js';

interface RequestLike {
  method: string;
  mode: string;
  url: string;
}

interface CacheLike {
  match: (request: RequestLike | string) => Promise<Response | undefined>;
  put: (request: RequestLike | string, response: Response) => Promise<void>;
  addAll: (urls: string[]) => Promise<void>;
}

interface CachesLike {
  open: (name: string) => Promise<CacheLike>;
  keys: () => Promise<string[]>;
  delete: (name: string) => Promise<boolean>;
}

type Listener = (event: unknown) => void;

interface Harness {
  cachesStore: Map<string, Map<string, Response>>;
  listeners: Map<string, Listener>;
  fetchMock: ReturnType<typeof vi.fn>;
  dispatchFetch: (request: RequestLike) => {
    respondWith: Promise<Response> | undefined;
    waitUntil: Promise<unknown>[];
  };
  dispatchLifecycle: (type: 'install' | 'activate') => Promise<unknown>[];
}

function keyOf(request: RequestLike | string): string {
  return typeof request === 'string' ? request : request.url;
}

function createHarness(fetchImpl: (request: RequestLike) => Promise<Response>): Harness {
  const cachesStore = new Map<string, Map<string, Response>>();
  const listeners = new Map<string, Listener>();
  const fetchMock = vi.fn(fetchImpl);

  const caches: CachesLike = {
    open: async (name) => {
      let store = cachesStore.get(name);
      if (!store) {
        store = new Map();
        cachesStore.set(name, store);
      }
      const current = store;
      return {
        match: async (request) => current.get(keyOf(request)),
        put: async (request, response) => {
          current.set(keyOf(request), response);
        },
        addAll: async (urls) => {
          for (const url of urls) {
            current.set(url, new Response('precache'));
          }
        },
      };
    },
    keys: async () => [...cachesStore.keys()],
    delete: async (name) => cachesStore.delete(name),
  };

  const self = {
    addEventListener: (type: string, handler: Listener) => {
      listeners.set(type, handler);
    },
    location: { origin: 'https://oximeal.test' },
    clients: { claim: vi.fn(async () => undefined) },
    skipWaiting: vi.fn(async () => undefined),
  };

  new Function('self', 'caches', 'fetch', 'Response', 'URL', source)(
    self,
    caches,
    fetchMock,
    Response,
    URL,
  );

  const dispatchFetch = (request: RequestLike) => {
    let respondWith: Promise<Response> | undefined;
    const waitUntil: Promise<unknown>[] = [];
    // Fiel a la spec: `waitUntil` solo puede invocarse durante el despacho
    // síncrono del evento y antes de que `respondWith` resuelva.
    let dispatchDone = false;
    let finished = false;

    const assertActive = () => {
      if (dispatchDone || finished) {
        throw new DOMException('El evento ya ha terminado', 'InvalidStateError');
      }
    };

    const event = {
      request,
      respondWith: (response: Promise<Response>) => {
        respondWith = response;
        void response.then(
          () => {
            finished = true;
          },
          () => {
            finished = true;
          },
        );
      },
      waitUntil: (promise: Promise<unknown>) => {
        assertActive();
        waitUntil.push(promise);
      },
    };

    listeners.get('fetch')?.(event);
    dispatchDone = true;

    return { respondWith, waitUntil };
  };

  const dispatchLifecycle = (type: 'install' | 'activate') => {
    const waitUntil: Promise<unknown>[] = [];
    const event = {
      waitUntil: (promise: Promise<unknown>) => {
        waitUntil.push(promise);
      },
    };
    listeners.get(type)?.(event);
    return waitUntil;
  };

  return { cachesStore, listeners, fetchMock, dispatchFetch, dispatchLifecycle };
}

function request(path: string, mode = 'cors', method = 'GET'): RequestLike {
  return { method, mode, url: `https://oximeal.test${path}` };
}

describe('sw.js', () => {
  it('no intercepta peticiones de la API', () => {
    const harness = createHarness(async () => new Response('api'));

    const { respondWith } = harness.dispatchFetch(request('/api/shopping-list'));

    expect(respondWith).toBeUndefined();
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('no intercepta peticiones de autenticación', () => {
    const harness = createHarness(async () => new Response('auth'));

    const { respondWith } = harness.dispatchFetch(request('/auth/me'));

    expect(respondWith).toBeUndefined();
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('no intercepta /api ni /auth sin barra final', () => {
    const harness = createHarness(async () => new Response('api'));

    expect(harness.dispatchFetch(request('/api')).respondWith).toBeUndefined();
    expect(harness.dispatchFetch(request('/auth')).respondWith).toBeUndefined();
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('no intercepta métodos distintos de GET', () => {
    const harness = createHarness(async () => new Response('post'));

    const { respondWith } = harness.dispatchFetch(request('/assets/app.js', 'cors', 'POST'));

    expect(respondWith).toBeUndefined();
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('no intercepta peticiones cross-origin', () => {
    const harness = createHarness(async () => new Response('otro'));

    const { respondWith } = harness.dispatchFetch({
      method: 'GET',
      mode: 'cors',
      url: 'https://otro.example/assets/app.js',
    });

    expect(respondWith).toBeUndefined();
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('sirve un asset con hash desde la red y lo cachea', async () => {
    const harness = createHarness(async () => new Response('js'));

    const { respondWith, waitUntil } = harness.dispatchFetch(request('/assets/app.js'));
    expect(respondWith).toBeDefined();
    const response = await respondWith;
    await Promise.all(waitUntil);

    expect(harness.fetchMock).toHaveBeenCalledTimes(1);
    expect(response?.ok).toBe(true);
    expect(harness.cachesStore.get(ASSET_CACHE)?.has(ASSET_URL)).toBe(true);
  });

  it('sirve un asset cacheado sin acudir a la red', async () => {
    const harness = createHarness(async () => {
      throw new Error('no debería llamarse a la red');
    });
    const cached = new Response('cacheado');
    harness.cachesStore.set(ASSET_CACHE, new Map([[ASSET_URL, cached]]));

    const { respondWith } = harness.dispatchFetch(request('/assets/app.js'));
    const response = await respondWith;

    expect(response).toBe(cached);
    expect(harness.fetchMock).not.toHaveBeenCalled();
  });

  it('devuelve Response.error() si el asset no está en caché y no hay red', async () => {
    const harness = createHarness(async () => {
      throw new Error('sin conexión');
    });

    const { respondWith } = harness.dispatchFetch(request('/assets/missing.js'));
    const response = await respondWith;

    expect(response?.ok).toBe(false);
    expect(response?.type).toBe('error');
  });

  it('sirve el shell desde caché y revalida en segundo plano (SWR)', async () => {
    const harness = createHarness(async () => new Response('red'));
    const cached = new Response('cacheado');
    const url = 'https://oximeal.test/manifest.webmanifest';
    harness.cachesStore.set(SHELL_CACHE, new Map([[url, cached]]));

    const { respondWith, waitUntil } = harness.dispatchFetch(request('/manifest.webmanifest'));
    const response = await respondWith;

    expect(response).toBe(cached);
    await Promise.all(waitUntil);

    const stored = harness.cachesStore.get(SHELL_CACHE)?.get(url);
    expect(stored).toBeDefined();
    expect(await stored?.text()).toBe('red');
  });

  it('cae al index.html cacheado cuando la navegación falla sin conexión', async () => {
    const harness = createHarness(async () => {
      throw new Error('sin conexión');
    });
    const cachedIndex = new Response('index cacheado');
    harness.cachesStore.set(SHELL_CACHE, new Map([['/index.html', cachedIndex]]));

    const { respondWith } = harness.dispatchFetch(request('/', 'navigate'));
    const response = await respondWith;

    expect(response).toBe(cachedIndex);
  });

  it('no cachea el index.html cuando la navegación responde 500', async () => {
    const harness = createHarness(async () => new Response('error', { status: 500 }));

    const { respondWith, waitUntil } = harness.dispatchFetch(request('/', 'navigate'));
    await respondWith;
    await Promise.all(waitUntil);

    expect(harness.cachesStore.get(SHELL_CACHE)?.has('/index.html')).not.toBe(true);
  });

  it('activate borra solo las cachés de OxiMeal y conserva las ajenas', async () => {
    const harness = createHarness(async () => new Response('ok'));
    harness.cachesStore.set('oximeal-shell-old', new Map());
    harness.cachesStore.set('oximeal-assets-old', new Map());
    harness.cachesStore.set('otra-app-cache', new Map());
    harness.cachesStore.set(SHELL_CACHE, new Map());
    harness.cachesStore.set(ASSET_CACHE, new Map());

    await Promise.all(harness.dispatchLifecycle('activate'));

    expect(harness.cachesStore.has('oximeal-shell-old')).toBe(false);
    expect(harness.cachesStore.has('oximeal-assets-old')).toBe(false);
    expect(harness.cachesStore.has('otra-app-cache')).toBe(true);
    expect(harness.cachesStore.has(SHELL_CACHE)).toBe(true);
    expect(harness.cachesStore.has(ASSET_CACHE)).toBe(true);
  });

  it('install precachea los assets del build', async () => {
    const harness = createHarness(async () => new Response('ok'));

    await Promise.all(harness.dispatchLifecycle('install'));

    expect(harness.cachesStore.get(ASSET_CACHE)?.has('/assets/app.js')).toBe(true);
    expect(harness.cachesStore.get(SHELL_CACHE)?.has('/index.html')).toBe(true);
  });
});
