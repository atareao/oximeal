import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../api/client';
import type { ShoppingItem } from '../api/types';
import {
  clientIdFromLocalId,
  configureMutationQueue,
  drainQueue,
  enqueueCreate,
  enqueueUpdate,
  getQueuedMutations,
  initMutationQueue,
  isLocalItemId,
  localItemId,
  parseServerTimestamp,
} from './mutationQueue';
import { createMutationStore, type MutationStore, type QueuedMutation } from './mutationStore';

function serverItem(overrides: Partial<ShoppingItem> & Pick<ShoppingItem, 'id'>): ShoppingItem {
  return {
    name: 'Leche',
    amount: 1,
    unit: 'ud',
    category: 'Refrigerados',
    checked: false,
    updated_at: '2026-11-01T10:00:00Z',
    ...overrides,
  };
}

describe('mutationQueue', () => {
  beforeEach(() => {
    vi.stubGlobal('indexedDB', undefined);
    configureMutationQueue(createMutationStore());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('explica los ids provisionales', () => {
    const { clientId, localId } = {
      clientId: 'abc',
      localId: localItemId('abc'),
    };

    expect(localId).toBe('local:abc');
    expect(isLocalItemId(localId)).toBe(true);
    expect(isLocalItemId('real-id')).toBe(false);
    expect(clientIdFromLocalId(localId)).toBe(clientId);
  });

  it('enqueueCreate devuelve los ids y deja la operación en la cola', async () => {
    const { clientId, localId } = await enqueueCreate({
      name: 'Leche',
      amount: 1,
      unit: 'ud',
      category: 'Refrigerados',
    });

    expect(localId).toBe(localItemId(clientId));
    const queue = getQueuedMutations();
    expect(queue).toHaveLength(1);
    expect(queue[0]).toMatchObject({
      kind: 'create',
      clientId,
      checked: null,
      item: { name: 'Leche', amount: 1, unit: 'ud', category: 'Refrigerados' },
    });
  });

  it('enqueueUpdate sobre un id local actualiza el checked del create y no añade operación', async () => {
    const { localId } = await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });

    await enqueueUpdate(localId, { checked: true });

    const queue = getQueuedMutations();
    expect(queue).toHaveLength(1);
    expect(queue[0]).toMatchObject({ kind: 'create', checked: true });
  });

  it('fusiona dos enqueueUpdate del mismo id real en una sola operación', async () => {
    await enqueueUpdate('real-1', { checked: true });
    await enqueueUpdate('real-1', { amount: 3 });

    const queue = getQueuedMutations();
    expect(queue).toHaveLength(1);
    expect(queue[0]).toMatchObject({
      kind: 'update',
      itemId: 'real-1',
      patch: { checked: true, amount: 3 },
    });
  });

  it('drena un create con toggle pendiente: POST y después PATCH', async () => {
    const { clientId, localId } = await enqueueCreate({
      name: 'Leche',
      amount: 1,
      unit: 'ud',
      category: 'Refrigerados',
    });
    await enqueueUpdate(localId, { checked: true });

    const createItem = vi.fn(async () => serverItem({ id: 'srv-1', checked: true }));
    const patchItem = vi.fn(async () => serverItem({ id: 'srv-1', checked: true }));
    const onCreated = vi.fn();

    const result = await drainQueue({ createItem, patchItem, onCreated });

    expect(createItem).toHaveBeenCalledWith({
      name: 'Leche',
      amount: 1,
      unit: 'ud',
      category: 'Refrigerados',
      client_id: clientId,
    });
    expect(patchItem).toHaveBeenCalledWith('srv-1', { checked: true });
    expect(onCreated).toHaveBeenCalledWith(clientId, expect.objectContaining({ id: 'srv-1' }));
    expect(result).toEqual({ processed: 1, discarded: 0, stoppedBy: null });
    expect(getQueuedMutations()).toHaveLength(0);
  });

  it('se detiene en el primer error de red y conserva las pendientes', async () => {
    await enqueueUpdate('a', { checked: true });
    await enqueueUpdate('b', { checked: true });

    const patchItem = vi.fn(async () => {
      throw new ApiError(0, 'network_error', 'sin red');
    });

    const result = await drainQueue({ createItem: vi.fn(), patchItem });

    expect(result.stoppedBy).toBe('network');
    expect(result.processed).toBe(0);
    expect(getQueuedMutations()).toHaveLength(2);
  });

  it('descarta un 4xx no recuperable, avisa y continúa con la siguiente', async () => {
    await enqueueUpdate('bad', { checked: true });
    await enqueueUpdate('good', { checked: true });

    const patchItem = vi.fn(async (id: string) => {
      if (id === 'bad') {
        throw new ApiError(422, 'invalid', 'no válido');
      }
      return serverItem({ id, checked: true });
    });
    const onDiscard = vi.fn();

    const result = await drainQueue({ createItem: vi.fn(), patchItem, onDiscard });

    expect(result.discarded).toBe(1);
    expect(result.processed).toBe(1);
    expect(result.stoppedBy).toBeNull();
    expect(onDiscard).toHaveBeenCalledTimes(1);
    expect(getQueuedMutations()).toHaveLength(0);
  });

  it('descarta la operación si la caché tiene un updated_at posterior (LWW)', async () => {
    await enqueueUpdate('it-1', { checked: true });

    const getCachedItem = vi.fn(() =>
      serverItem({
        id: 'it-1',
        checked: false,
        updated_at: new Date(Date.now() + 60_000).toISOString(),
      }),
    );
    const patchItem = vi.fn();
    const onDiscard = vi.fn();

    const result = await drainQueue({ createItem: vi.fn(), patchItem, getCachedItem, onDiscard });

    expect(patchItem).not.toHaveBeenCalled();
    expect(onDiscard).not.toHaveBeenCalled();
    expect(result.processed).toBe(0);
    expect(getQueuedMutations()).toHaveLength(0);
  });

  it('aplica el patch si la caché no es más reciente', async () => {
    await enqueueUpdate('it-1', { checked: true });

    const getCachedItem = vi.fn(() =>
      serverItem({ id: 'it-1', updated_at: '2026-11-01T10:00:00Z' }),
    );
    const patchItem = vi.fn(async (id: string) => serverItem({ id, checked: true }));

    const result = await drainQueue({ createItem: vi.fn(), patchItem, getCachedItem });

    expect(patchItem).toHaveBeenCalledWith('it-1', { checked: true });
    expect(result.processed).toBe(1);
  });

  it('dos drenados concurrentes comparten la misma promesa', async () => {
    await enqueueUpdate('it-1', { checked: true });

    let resolvePatch: ((item: ShoppingItem) => void) | undefined;
    const patchItem = vi.fn(
      () =>
        new Promise<ShoppingItem>((resolve) => {
          resolvePatch = resolve;
        }),
    );

    const first = drainQueue({ createItem: vi.fn(), patchItem });
    const second = drainQueue({ createItem: vi.fn(), patchItem });

    expect(first).toBe(second);

    resolvePatch?.(serverItem({ id: 'it-1', checked: true }));
    await first;

    expect(patchItem).toHaveBeenCalledTimes(1);
  });

  it('initMutationQueue con un store que rechaza deja la cola vacía sin lanzar', async () => {
    const failingStore: MutationStore = {
      readMutations: vi.fn(() => Promise.reject(new Error('boom'))),
      addMutation: vi.fn(),
      putMutation: vi.fn(),
      deleteMutation: vi.fn(),
      clearMutations: vi.fn(),
    };
    configureMutationQueue(failingStore);

    await expect(initMutationQueue()).resolves.toBeUndefined();
    expect(getQueuedMutations()).toEqual([]);
  });

  it('recarga la cola persistida en orden de seq', async () => {
    const persisted: QueuedMutation[] = [
      { seq: 1, kind: 'update', itemId: 'a', patch: { checked: true }, updatedAt: 1 },
      { seq: 2, kind: 'update', itemId: 'b', patch: { checked: false }, updatedAt: 2 },
    ];
    const store: MutationStore = {
      readMutations: vi.fn(async () => persisted),
      addMutation: vi.fn(async () => 0),
      putMutation: vi.fn(async () => undefined),
      deleteMutation: vi.fn(async () => undefined),
      clearMutations: vi.fn(async () => undefined),
    };
    configureMutationQueue(store);

    await initMutationQueue();

    expect(
      getQueuedMutations().map((mutation) =>
        mutation.kind === 'update' ? mutation.itemId : mutation.clientId,
      ),
    ).toEqual(['a', 'b']);
  });

  it('un 401 durante el drenado se detiene con stoppedBy auth y conserva la operación', async () => {
    await enqueueUpdate('it-1', { checked: true });

    const patchItem = vi.fn(async () => {
      throw new ApiError(401, 'unauthorized', 'sin sesión');
    });

    const result = await drainQueue({ createItem: vi.fn(), patchItem });

    expect(result.stoppedBy).toBe('auth');
    expect(result.processed).toBe(0);
    expect(getQueuedMutations()).toHaveLength(1);
  });

  it('aplica un amount pendiente sobre un id local tras el alta', async () => {
    const { localId } = await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });
    await enqueueUpdate(localId, { amount: 5 });

    const createItem = vi.fn(async () => serverItem({ id: 'srv-1' }));
    const patchItem = vi.fn(async () => serverItem({ id: 'srv-1' }));

    const result = await drainQueue({ createItem, patchItem });

    expect(patchItem).toHaveBeenCalledWith('srv-1', { amount: 5 });
    expect(result.processed).toBe(1);
    expect(getQueuedMutations()).toHaveLength(0);
  });

  it('enqueueUpdate sobre un id local fusiona checked y amount en el create', async () => {
    const { localId } = await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });

    await enqueueUpdate(localId, { checked: true });
    await enqueueUpdate(localId, { amount: 3 });

    const queue = getQueuedMutations();
    expect(queue).toHaveLength(1);
    expect(queue[0]).toMatchObject({ kind: 'create', checked: true, amount: 3 });
  });

  it('descarta la operación por LWW con un updated_at en formato SQLite (UTC sin zona)', async () => {
    await enqueueUpdate('it-1', { checked: true });

    // `CURRENT_TIMESTAMP` de SQLite: UTC sin `T` ni `Z`, un minuto en el futuro.
    const future = new Date(Date.now() + 60_000).toISOString().replace('T', ' ').slice(0, 19);
    const getCachedItem = vi.fn(() => serverItem({ id: 'it-1', updated_at: future }));
    const patchItem = vi.fn();

    const result = await drainQueue({ createItem: vi.fn(), patchItem, getCachedItem });

    expect(patchItem).not.toHaveBeenCalled();
    expect(result.processed).toBe(0);
    expect(getQueuedMutations()).toHaveLength(0);
  });
});

describe('parseServerTimestamp', () => {
  it('interpreta el formato de SQLite (sin zona) como UTC', () => {
    expect(parseServerTimestamp('2026-11-01 10:00:00')).toBe(Date.UTC(2026, 10, 1, 10, 0, 0));
  });

  it('acepta ISO-8601 con marca Z', () => {
    expect(parseServerTimestamp('2026-11-01T10:00:00.000Z')).toBe(Date.UTC(2026, 10, 1, 10, 0, 0));
  });

  it('respeta un offset explícito', () => {
    expect(parseServerTimestamp('2026-11-01T10:00:00+02:00')).toBe(Date.UTC(2026, 10, 1, 8, 0, 0));
  });

  it('devuelve null si el texto no es una fecha válida', () => {
    expect(parseServerTimestamp('no-es-fecha')).toBeNull();
  });
});
