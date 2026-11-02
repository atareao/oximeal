import { StrictMode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, render } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { createShoppingItem } from '../api/client';
import type { ShoppingItem, ShoppingItemInput } from '../api/types';
import { MutationQueueProvider } from './MutationQueueProvider';
import { configureMutationQueue, enqueueCreate, getQueuedMutations } from './mutationQueue';
import { createMutationStore } from './mutationStore';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return { ...actual, createShoppingItem: vi.fn(), patchShoppingItem: vi.fn() };
});

const mockedCreate = vi.mocked(createShoppingItem);

function newQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Infinity } },
  });
}

/** Ítem que el servidor devuelve tras un alta satisfactoria. */
function serverItem(input: ShoppingItemInput, id = 'srv-1'): ShoppingItem {
  return {
    id,
    name: input.name,
    amount: input.amount,
    unit: input.unit,
    category: input.category ?? 'otros',
    checked: false,
    updated_at: new Date().toISOString(),
    client_id: input.client_id ?? null,
  };
}

/**
 * Avanza el tiempo de los temporizadores falsos dejando que React y las
 * promesas del drenado se asienten dentro de `act`.
 */
async function advance(ms: number): Promise<void> {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(ms);
  });
}

/** Renderiza el provider (opcionalmente bajo StrictMode) dentro de react-query. */
function renderProvider(strict = false): ReturnType<typeof render> {
  const queryClient = newQueryClient();
  const tree = (
    <QueryClientProvider client={queryClient}>
      <MutationQueueProvider />
    </QueryClientProvider>
  );
  return render(strict ? <StrictMode>{tree}</StrictMode> : tree);
}

describe('MutationQueueProvider (drenado con reintento)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    vi.stubGlobal('indexedDB', undefined);
    configureMutationQueue(createMutationStore());
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('reintenta solo y termina drenando la cola cuando la red vuelve', async () => {
    let attempts = 0;
    mockedCreate.mockImplementation(async (input) => {
      attempts += 1;
      if (attempts <= 2) {
        throw new TypeError('Failed to fetch');
      }
      return serverItem(input);
    });

    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'l' });
    expect(getQueuedMutations()).toHaveLength(1);

    renderProvider();

    // Arranque: init + primer drenado (fallo de red) → reintento programado.
    await advance(0);
    expect(mockedCreate).toHaveBeenCalled();
    const callsAfterStart = mockedCreate.mock.calls.length;

    // Reintentos con backoff: el servidor acaba respondiendo y la cola se
    // vacía sola.
    await advance(2000);
    await advance(4000);
    await advance(60_000);

    expect(getQueuedMutations()).toHaveLength(0);
    expect(mockedCreate.mock.calls.length).toBeGreaterThan(callsAfterStart);

    // Sin pendientes, no hay más reintentos.
    const callsAfterDrain = mockedCreate.mock.calls.length;
    await advance(60_000);
    expect(mockedCreate.mock.calls.length).toBe(callsAfterDrain);
  });

  it('bajo StrictMode la cola se drena sola cuando la red vuelve', async () => {
    let attempts = 0;
    mockedCreate.mockImplementation(async (input) => {
      attempts += 1;
      if (attempts <= 2) {
        throw new TypeError('Failed to fetch');
      }
      return serverItem(input);
    });

    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'l' });
    expect(getQueuedMutations()).toHaveLength(1);

    // StrictMode monta-desmonta-monta los efectos. Con el planificador atado al
    // montaje (creado en el efecto dueño), cada montaje tiene el suyo y la cola
    // se sigue drenando. Con el planificador compartido, el `dispose()` del
    // primer desmontaje lo dejaría inutilizado y la cola nunca se drenaría.
    renderProvider(true);

    await advance(0);
    await advance(2000);
    await advance(4000);
    await advance(60_000);

    expect(getQueuedMutations()).toHaveLength(0);
    expect(mockedCreate.mock.calls.length).toBeGreaterThanOrEqual(3);
  });

  it('drena de inmediato al recuperar la visibilidad, sin avanzar temporizadores', async () => {
    mockedCreate.mockImplementation(async () => {
      throw new TypeError('Failed to fetch');
    });

    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'l' });

    const visibilityState = vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');

    renderProvider();

    // Primer drenado tras el arranque (fallo de red) → reintento programado.
    await advance(0);
    const callsAfterMount = mockedCreate.mock.calls.length;
    expect(callsAfterMount).toBeGreaterThanOrEqual(1);

    // Sin avanzar el temporizador, el evento de visibilidad fuerza un intento.
    await act(async () => {
      document.dispatchEvent(new Event('visibilitychange'));
    });
    expect(mockedCreate.mock.calls.length).toBeGreaterThan(callsAfterMount);

    visibilityState.mockRestore();
  });

  it('drena de inmediato al recuperar el foco, sin avanzar temporizadores', async () => {
    mockedCreate.mockImplementation(async () => {
      throw new TypeError('Failed to fetch');
    });

    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'l' });

    renderProvider();

    await advance(0);
    const callsAfterMount = mockedCreate.mock.calls.length;
    expect(callsAfterMount).toBeGreaterThanOrEqual(1);

    await act(async () => {
      window.dispatchEvent(new Event('focus'));
    });
    expect(mockedCreate.mock.calls.length).toBeGreaterThan(callsAfterMount);
  });

  it('al desmontar cancela los temporizadores y no vuelve a drenar', async () => {
    mockedCreate.mockImplementation(async () => {
      throw new TypeError('Failed to fetch');
    });

    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'l' });

    const { unmount } = renderProvider();

    // Tras el fallo de red queda un reintento programado.
    await advance(0);
    expect(vi.getTimerCount()).toBeGreaterThan(0);
    const callsBeforeUnmount = mockedCreate.mock.calls.length;

    unmount();

    // El `dispose()` del desmontaje limpia el temporizador pendiente.
    expect(vi.getTimerCount()).toBe(0);

    await advance(60_000);
    expect(mockedCreate.mock.calls.length).toBe(callsBeforeUnmount);
  });
});
