import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createIdbStore } from './idb';

/**
 * jsdom no implementa IndexedDB. El wrapper debe degradar con elegancia:
 * nunca lanza y se comporta como un store vacío.
 */
describe('createIdbStore sin IndexedDB', () => {
  beforeEach(() => {
    vi.stubGlobal('indexedDB', undefined);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('no lanza y devuelve una lista vacía al leer', async () => {
    expect(typeof indexedDB).toBe('undefined');

    const store = createIdbStore();

    await expect(store.readQueries()).resolves.toEqual([]);
  });

  it('writeQueries, replaceQueries y clearQueries son no-op', async () => {
    const store = createIdbStore();

    await expect(
      store.writeQueries([{ key: '["shopping-list"]', state: {}, updatedAt: 1 }]),
    ).resolves.toBeUndefined();
    await expect(
      store.replaceQueries([{ key: '["shopping-list"]', state: {}, updatedAt: 1 }]),
    ).resolves.toBe(false);
    await expect(store.clearQueries()).resolves.toBeUndefined();
  });
});
