import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import { ApiError, patchShoppingItem } from '../api/client';
import { configureMutationQueue, enqueueUpdate } from './mutationQueue';
import { createMutationStore } from './mutationStore';
import { MutationQueueNotifier } from './MutationQueueNotifier';
import { syncShoppingQueue } from './syncShoppingQueue';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return { ...actual, createShoppingItem: vi.fn(), patchShoppingItem: vi.fn() };
});

const mockedPatch = vi.mocked(patchShoppingItem);

describe('MutationQueueNotifier', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal('indexedDB', undefined);
    configureMutationQueue(createMutationStore());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('avisa por message.error cuando se descarta una operación (4xx)', async () => {
    render(
      <AntdApp>
        <MutationQueueNotifier />
      </AntdApp>,
    );

    await enqueueUpdate('bad', { checked: true });
    mockedPatch.mockRejectedValue(new ApiError(422, 'invalid', 'no válido'));

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Infinity } },
    });
    await syncShoppingQueue(queryClient);

    expect(await screen.findByText(/Se descartó un cambio pendiente/)).toBeInTheDocument();
  });
});
