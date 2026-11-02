import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { QueryClient } from '@tanstack/react-query';
import { ApiError, createShoppingItem, patchShoppingItem } from '../api/client';
import { configureMutationQueue, enqueueUpdate } from './mutationQueue';
import { createMutationStore } from './mutationStore';
import { setDiscardHandler, syncShoppingQueue } from './syncShoppingQueue';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return { ...actual, createShoppingItem: vi.fn(), patchShoppingItem: vi.fn() };
});

const mockedCreate = vi.mocked(createShoppingItem);
const mockedPatch = vi.mocked(patchShoppingItem);

function newQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Infinity } },
  });
}

describe('syncShoppingQueue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal('indexedDB', undefined);
    configureMutationQueue(createMutationStore());
  });

  afterEach(() => {
    setDiscardHandler(null);
    vi.unstubAllGlobals();
  });

  it('el descarte por 4xx invoca el handler registrado', async () => {
    const handler = vi.fn();
    setDiscardHandler(handler);

    await enqueueUpdate('bad', { checked: true });
    mockedPatch.mockRejectedValue(new ApiError(422, 'invalid', 'no válido'));

    const result = await syncShoppingQueue(newQueryClient());

    expect(result.discarded).toBe(1);
    expect(mockedCreate).not.toHaveBeenCalled();
    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith(expect.any(ApiError));
  });

  it('sin handler registrado el descarte no lanza', async () => {
    setDiscardHandler(null);
    await enqueueUpdate('bad', { checked: true });
    mockedPatch.mockRejectedValue(new ApiError(422, 'invalid', 'no válido'));

    await expect(syncShoppingQueue(newQueryClient())).resolves.toMatchObject({ discarded: 1 });
  });

  it('con la cola vacía no lanza ningún drenado', async () => {
    const result = await syncShoppingQueue(newQueryClient());

    expect(result).toEqual({ processed: 0, discarded: 0, stoppedBy: null });
    expect(mockedPatch).not.toHaveBeenCalled();
  });
});
