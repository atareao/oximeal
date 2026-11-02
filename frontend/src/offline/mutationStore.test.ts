import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createMutationStore, type QueuedMutation } from './mutationStore';

type CreateMutation = Extract<QueuedMutation, { kind: 'create' }>;

/**
 * jsdom no implementa IndexedDB. El store debe degradar con elegancia a una
 * cola en memoria: nunca lanza y mantiene las mismas garantías.
 */
describe('createMutationStore sin IndexedDB', () => {
  beforeEach(() => {
    vi.stubGlobal('indexedDB', undefined);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function createMutation(clientId = 'client-1'): CreateMutation {
    return {
      kind: 'create',
      clientId,
      item: { name: 'Leche', amount: 1, unit: 'ud' },
      checked: null,
      amount: null,
      createdAt: 1,
    };
  }

  it('funciona en memoria: añade y lee ordenado por seq', async () => {
    const store = createMutationStore();

    const first = await store.addMutation(createMutation());
    const second = await store.addMutation(createMutation('client-2'));

    expect(second).toBeGreaterThan(first);

    const mutations = await store.readMutations();
    expect(mutations).toHaveLength(2);
    expect(mutations.map((mutation) => mutation.seq)).toEqual([first, second]);
    expect(mutations[0]).toMatchObject({ kind: 'create', clientId: 'client-1' });
  });

  it('borra por seq y limpia la cola', async () => {
    const store = createMutationStore();
    const first = await store.addMutation(createMutation());
    const second = await store.addMutation(createMutation('client-2'));

    await store.deleteMutation(first);
    await expect(store.readMutations()).resolves.toEqual([
      expect.objectContaining({ seq: second }),
    ]);

    await store.clearMutations();
    await expect(store.readMutations()).resolves.toEqual([]);
  });

  it('hace upsert por seq con putMutation', async () => {
    const store = createMutationStore();
    const seq = await store.addMutation(createMutation());

    await store.putMutation({ ...createMutation(), seq, checked: true });

    const mutations = await store.readMutations();
    expect(mutations).toHaveLength(1);
    expect(mutations[0]).toMatchObject({ seq, checked: true });
  });
});
