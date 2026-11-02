import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { QueryClient, dehydrate } from '@tanstack/react-query';
import type { CachedQuery, OfflineStore } from './idb';
import {
  PERSISTED_QUERY_ROOTS,
  clearPersistedQueries,
  hasCachedData,
  hydrateQueryClient,
  isPersistedQueryKey,
  persistQueryClient,
} from './queryPersist';

/** Store en memoria: evita depender de `fake-indexeddb`. */
function createMemoryStore(): OfflineStore & { entries: CachedQuery[] } {
  const entries: CachedQuery[] = [];
  return {
    entries,
    async readQueries() {
      return entries.map((entry) => ({ ...entry }));
    },
    async writeQueries(next) {
      for (const entry of next) {
        const index = entries.findIndex((candidate) => candidate.key === entry.key);
        if (index >= 0) {
          entries[index] = entry;
        } else {
          entries.push(entry);
        }
      }
    },
    async replaceQueries(next) {
      entries.length = 0;
      for (const entry of next) {
        entries.push(entry);
      }
      return true;
    },
    async clearQueries() {
      entries.length = 0;
    },
  };
}

/** Serializa la caché de un cliente como lo haría `persistQueryClient`. */
function snapshot(client: QueryClient): CachedQuery[] {
  const state = dehydrate(client, {
    shouldDehydrateQuery: (query) => isPersistedQueryKey(query.queryKey),
  });
  return state.queries.map((query) => ({
    key: query.queryHash,
    state: query,
    updatedAt: Date.now(),
  }));
}

const originalVisibilityState = Object.getOwnPropertyDescriptor(
  Document.prototype,
  'visibilityState',
);

describe('isPersistedQueryKey', () => {
  it('acepta las raíces de la lista blanca', () => {
    for (const root of PERSISTED_QUERY_ROOTS) {
      expect(isPersistedQueryKey([root])).toBe(true);
      expect(isPersistedQueryKey([root, 'detalle', 1])).toBe(true);
    }
  });

  it('rechaza claves fuera de la lista blanca y claves vacías', () => {
    expect(isPersistedQueryKey(['auth'])).toBe(false);
    expect(isPersistedQueryKey([])).toBe(false);
    expect(isPersistedQueryKey([42])).toBe(false);
  });
});

describe('hydrateQueryClient', () => {
  beforeEach(async () => {
    await clearPersistedQueries(createMemoryStore());
  });

  it('rellena la caché de un cliente nuevo y devuelve true', async () => {
    const source = new QueryClient();
    source.setQueryData(['shopping-list'], [{ id: '1', name: 'Leche' }]);

    const store = createMemoryStore();
    await store.replaceQueries(snapshot(source));

    const target = new QueryClient();
    const hydrated = await hydrateQueryClient(target, store);

    expect(hydrated).toBe(true);
    expect(target.getQueryData(['shopping-list'])).toEqual([{ id: '1', name: 'Leche' }]);
    expect(hasCachedData()).toBe(true);
  });

  it('devuelve false cuando el store está vacío', async () => {
    const target = new QueryClient();
    const hydrated = await hydrateQueryClient(target, createMemoryStore());

    expect(hydrated).toBe(false);
    expect(hasCachedData()).toBe(false);
  });

  it('devuelve false y no lanza si readQueries rechaza', async () => {
    const failing: OfflineStore = {
      readQueries: () => Promise.reject(new Error('boom')),
      writeQueries: async () => undefined,
      replaceQueries: async () => false,
      clearQueries: async () => undefined,
    };

    await expect(hydrateQueryClient(new QueryClient(), failing)).resolves.toBe(false);
    expect(hasCachedData()).toBe(false);
  });

  it('ignora las entradas fuera de la lista blanca', async () => {
    const store = createMemoryStore();
    store.entries.push(
      {
        key: '["auth","me"]',
        state: { queryKey: ['auth', 'me'], state: { data: { name: 'Ana' } } },
        updatedAt: Date.now(),
      },
      {
        key: '["recipes","list",{}]',
        state: { queryKey: ['recipes', 'list', {}], state: { data: [] } },
        updatedAt: Date.now(),
      },
    );

    const target = new QueryClient();
    const hydrated = await hydrateQueryClient(target, store);

    expect(hydrated).toBe(true);
    expect(target.getQueryData(['auth', 'me'])).toBeUndefined();
    expect(target.getQueryData(['recipes', 'list', {}])).toEqual([]);
  });
});

describe('persistQueryClient', () => {
  beforeEach(async () => {
    await clearPersistedQueries(createMemoryStore());
  });

  afterEach(() => {
    vi.useRealTimers();
    if (originalVisibilityState) {
      Object.defineProperty(document, 'visibilityState', originalVisibilityState);
    }
  });

  it('escribe en el store tras un cambio de datos (debounce)', async () => {
    vi.useFakeTimers();
    const client = new QueryClient();
    const store = createMemoryStore();
    const unsubscribe = persistQueryClient(client, store);

    client.setQueryData(['shopping-list'], [{ id: '1' }]);
    expect(store.entries).toHaveLength(0);

    await vi.advanceTimersByTimeAsync(300);

    expect(store.entries).toHaveLength(1);
    expect(store.entries[0]?.key).toContain('shopping-list');
    expect(hasCachedData()).toBe(true);

    unsubscribe();
  });

  it('solo persiste las claves de PERSISTED_QUERY_ROOTS', async () => {
    vi.useFakeTimers();
    const client = new QueryClient();
    const store = createMemoryStore();
    const unsubscribe = persistQueryClient(client, store);

    client.setQueryData(['shopping-list'], []);
    client.setQueryData(['auth', 'me'], { name: 'Ana' });
    client.setQueryData(['recipes', 'list', {}], []);

    await vi.advanceTimersByTimeAsync(300);

    expect(store.entries).toHaveLength(2);
    expect(store.entries.map((entry) => entry.key).join(' ')).not.toContain('auth');

    unsubscribe();
  });

  it('persiste al ocultar la pestaña sin esperar al debounce', async () => {
    const client = new QueryClient();
    const store = createMemoryStore();
    const unsubscribe = persistQueryClient(client, store);

    client.setQueryData(['meal-plan', '2026-01-01', '2026-01-07'], []);
    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      get: () => 'hidden',
    });
    document.dispatchEvent(new Event('visibilitychange'));

    expect(store.entries).toHaveLength(1);

    unsubscribe();
  });

  it('hasCachedData pasa a false tras persistir cero entradas', async () => {
    vi.useFakeTimers();

    const source = new QueryClient();
    source.setQueryData(['shopping-list'], [{ id: '1' }]);
    const store = createMemoryStore();
    await store.replaceQueries(snapshot(source));
    await hydrateQueryClient(new QueryClient(), store);
    expect(hasCachedData()).toBe(true);

    const client = new QueryClient();
    const unsubscribe = persistQueryClient(client, store);
    client.setQueryData(['auth', 'me'], { name: 'Ana' });
    await vi.advanceTimersByTimeAsync(300);

    expect(hasCachedData()).toBe(false);
    expect(store.entries).toHaveLength(0);

    unsubscribe();
  });

  it('replaceQueries elimina las claves que ya no están presentes', async () => {
    vi.useFakeTimers();
    const client = new QueryClient();
    const store = createMemoryStore();
    const unsubscribe = persistQueryClient(client, store);

    client.setQueryData(['shopping-list'], [{ id: '1' }]);
    client.setQueryData(['recipes', 'list', {}], [{ id: 'r1' }]);
    await vi.advanceTimersByTimeAsync(300);
    expect(store.entries).toHaveLength(2);

    client.removeQueries({ queryKey: ['recipes'] });
    await vi.advanceTimersByTimeAsync(300);

    expect(store.entries).toHaveLength(1);
    expect(store.entries[0]?.key).toContain('shopping-list');

    unsubscribe();
  });

  it('unsubscribe elimina los listeners de visibilitychange y pagehide', () => {
    const removeDocumentListener = vi.spyOn(document, 'removeEventListener');
    const removeWindowListener = vi.spyOn(window, 'removeEventListener');
    const unsubscribe = persistQueryClient(new QueryClient(), createMemoryStore());

    unsubscribe();

    expect(removeDocumentListener).toHaveBeenCalledWith('visibilitychange', expect.any(Function));
    expect(removeWindowListener).toHaveBeenCalledWith('pagehide', expect.any(Function));
  });
});

describe('clearPersistedQueries', () => {
  it('vacía el store y deja hasCachedData en false', async () => {
    const source = new QueryClient();
    source.setQueryData(['shopping-list'], [{ id: '1' }]);
    const store = createMemoryStore();
    await store.replaceQueries(snapshot(source));

    await hydrateQueryClient(new QueryClient(), store);
    expect(hasCachedData()).toBe(true);

    await clearPersistedQueries(store);

    expect(store.entries).toHaveLength(0);
    expect(hasCachedData()).toBe(false);
  });
});
