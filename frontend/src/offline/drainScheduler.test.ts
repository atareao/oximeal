import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DrainResult } from './mutationQueue';
import { createDrainScheduler } from './drainScheduler';

function network(): DrainResult {
  return { processed: 0, discarded: 0, stoppedBy: 'network' };
}

describe('createDrainScheduler', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('reintenta con espera creciente mientras el drenado se detiene por red', async () => {
    const drain = vi.fn().mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(1);

    // Deja resolver el drenado y programar el primer reintento (2000 ms).
    await vi.advanceTimersByTimeAsync(0);
    expect(drain).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(2000);
    expect(drain).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(4000);
    expect(drain).toHaveBeenCalledTimes(3);

    await vi.advanceTimersByTimeAsync(8000);
    expect(drain).toHaveBeenCalledTimes(4);

    scheduler.dispose();
  });

  it('acota la espera al máximo del backoff', async () => {
    const drain = vi.fn().mockResolvedValue(network());
    const scheduler = createDrainScheduler({
      drain,
      hasPending: () => true,
      backoffMs: [10, 20, 30],
    });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(0); // programa 10
    expect(drain).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(9);
    expect(drain).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1); // dispara 1º → programa 20
    expect(drain).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(20); // dispara 2º → programa 30
    expect(drain).toHaveBeenCalledTimes(3);

    await vi.advanceTimersByTimeAsync(30); // dispara 3º → programa 30 (acotado)
    expect(drain).toHaveBeenCalledTimes(4);

    // La siguiente espera es 30, nunca mayor.
    await vi.advanceTimersByTimeAsync(29);
    expect(drain).toHaveBeenCalledTimes(4);
    await vi.advanceTimersByTimeAsync(1);
    expect(drain).toHaveBeenCalledTimes(5);

    scheduler.dispose();
  });

  it('con la cola vacía no programa reintento y reinicia el backoff', async () => {
    let pending = true;
    const drain = vi.fn(async (): Promise<DrainResult> => {
      if (pending) {
        return network();
      }
      return { processed: 1, discarded: 0, stoppedBy: null };
    });
    const scheduler = createDrainScheduler({ drain, hasPending: () => pending });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(0); // programa 2000, attempt = 1
    expect(drain).toHaveBeenCalledTimes(1);

    // La cola se vacía antes del reintento (p. ej. otro drenado la vació).
    pending = false;
    await vi.advanceTimersByTimeAsync(2000);
    expect(drain).toHaveBeenCalledTimes(2);

    // No hay más llamadas: la cola quedó vacía.
    await vi.advanceTimersByTimeAsync(1_000_000);
    expect(drain).toHaveBeenCalledTimes(2);

    // Se repone la cola: el siguiente fallo de red espera el mínimo (2000),
    // no un valor arrastrado del ciclo anterior.
    pending = true;
    scheduler.run();
    await vi.advanceTimersByTimeAsync(0);
    expect(drain).toHaveBeenCalledTimes(3);

    await vi.advanceTimersByTimeAsync(1999);
    expect(drain).toHaveBeenCalledTimes(3);
    await vi.advanceTimersByTimeAsync(1);
    expect(drain).toHaveBeenCalledTimes(4);

    scheduler.dispose();
  });

  it('reintenta de inmediato cuando el drenado progresó y quedan pendientes', async () => {
    const drain = vi
      .fn()
      .mockResolvedValueOnce({ processed: 1, discarded: 0, stoppedBy: 'network' })
      .mockResolvedValue({ processed: 0, discarded: 0, stoppedBy: 'auth' });
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(1);

    // Progreso + pendientes ⇒ reprograma con demora 0.
    await vi.advanceTimersByTimeAsync(0);
    expect(drain).toHaveBeenCalledTimes(2);

    // El segundo intento se detiene por auth: sin más reintentos.
    await vi.advanceTimersByTimeAsync(1_000_000);
    expect(drain).toHaveBeenCalledTimes(2);

    scheduler.dispose();
  });

  it('no reintenta en bucle ante un 401', async () => {
    const drain = vi.fn().mockResolvedValue({ processed: 0, discarded: 0, stoppedBy: 'auth' });
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(1_000_000);

    expect(drain).toHaveBeenCalledTimes(1);

    scheduler.dispose();
  });

  it('dispose cancela el temporizador y neutraliza futuros run', async () => {
    const drain = vi.fn().mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(0);
    expect(drain).toHaveBeenCalledTimes(1);

    scheduler.dispose();

    await vi.advanceTimersByTimeAsync(1_000_000);
    expect(drain).toHaveBeenCalledTimes(1);

    // Un `run` posterior es no-op.
    scheduler.run();
    await vi.advanceTimersByTimeAsync(1_000_000);
    expect(drain).toHaveBeenCalledTimes(1);
  });

  it('usa setTimeout (reprogramado) y nunca setInterval', async () => {
    const setTimeoutSpy = vi.spyOn(globalThis, 'setTimeout');
    const setIntervalSpy = vi.spyOn(globalThis, 'setInterval');
    const drain = vi.fn().mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(0);
    await vi.advanceTimersByTimeAsync(2000);
    await vi.advanceTimersByTimeAsync(4000);

    expect(setTimeoutSpy).toHaveBeenCalled();
    expect(setIntervalSpy).not.toHaveBeenCalled();
    expect(vi.getTimerCount()).toBeGreaterThan(0);

    scheduler.dispose();
  });

  it('reintenta con backoff si el drenado rechaza y quedan pendientes', async () => {
    const drain = vi.fn().mockRejectedValueOnce(new Error('boom')).mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(1);

    // Deja asentar el rechazo y programar el backoff (2000 ms).
    await vi.advanceTimersByTimeAsync(0);
    expect(drain).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(1999);
    expect(drain).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(drain).toHaveBeenCalledTimes(2);

    scheduler.dispose();
  });

  it('run() durante un drenado en vuelo fuerza un intento inmediato al terminar', async () => {
    let resolveFirst: ((result: DrainResult) => void) | undefined;
    const first = new Promise<DrainResult>((resolve) => {
      resolveFirst = resolve;
    });
    const drain = vi.fn().mockReturnValueOnce(first).mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(1);

    // Coalescido: no arranca otro drenado, pero deja la intención marcada.
    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(1);

    // El primero termina como fallo de red con pendientes.
    resolveFirst?.(network());
    await vi.advanceTimersByTimeAsync(0);

    // La petición coalescida se dispara con demora 0 (sin esperar 2000 ms).
    expect(drain).toHaveBeenCalledTimes(2);

    scheduler.dispose();
  });

  it('run fuerza un intento inmediato y reinicia el backoff', async () => {
    const drain = vi.fn().mockResolvedValue(network());
    const scheduler = createDrainScheduler({ drain, hasPending: () => true });

    scheduler.run();
    await vi.advanceTimersByTimeAsync(0); // programa 2000
    await vi.advanceTimersByTimeAsync(2000); // 2º → programa 4000
    await vi.advanceTimersByTimeAsync(4000); // 3º → programa 8000
    expect(drain).toHaveBeenCalledTimes(3);

    // run() dispara ya, sin esperar al temporizador de 8000.
    scheduler.run();
    expect(drain).toHaveBeenCalledTimes(4);

    // Y el backoff vuelve al mínimo: el siguiente reintento es 2000, no 16000.
    await vi.advanceTimersByTimeAsync(0);
    await vi.advanceTimersByTimeAsync(1999);
    expect(drain).toHaveBeenCalledTimes(4);
    await vi.advanceTimersByTimeAsync(1);
    expect(drain).toHaveBeenCalledTimes(5);

    scheduler.dispose();
  });
});
