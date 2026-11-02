/**
 * Planificador del reintento del drenado de la cola de escrituras offline.
 *
 * Aísla la política de reintento del provider para poder testearla con
 * temporizadores falsos. Reintenta con backoff exponencial acotado mientras el
 * drenado se detenga por un fallo de red, reprograma de inmediato si progresó y
 * aún quedan pendientes, y no reintenta en bucle ante 401/403 (se espera a que
 * cambien las condiciones, que vuelven a invocar `run`).
 *
 * Usa un `setTimeout` reprogramado, nunca un `setInterval`, y garantiza un solo
 * drenado en vuelo.
 */

import type { DrainResult } from './mutationQueue';

/** Retrasos del backoff exponencial acotado (ms). */
export const DEFAULT_BACKOFF_MS = [2000, 4000, 8000, 16000, 30000] as const;

export interface DrainSchedulerOptions {
  /** Ejecuta el drenado; la promesa resuelve con su resultado. */
  drain: () => Promise<DrainResult>;
  /** ¿Quedan operaciones pendientes? */
  hasPending: () => boolean;
  /** Retrasos del backoff (por defecto `DEFAULT_BACKOFF_MS`). */
  backoffMs?: readonly number[];
}

export interface DrainScheduler {
  /** Fuerza un intento inmediato y reinicia el backoff. */
  run(): void;
  /** Cancela el temporizador pendiente y libera el planificador. */
  dispose(): void;
}

/** Crea un planificador de drenado con reintento por red. */
export function createDrainScheduler(options: DrainSchedulerOptions): DrainScheduler {
  const backoff =
    options.backoffMs && options.backoffMs.length > 0 ? options.backoffMs : DEFAULT_BACKOFF_MS;

  let attempt = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let draining = false;
  /**
   * Un `run()` llegó mientras había un drenado en vuelo. Al terminar este se
   * fuerza un intento inmediato (demora 0) para no perder esa petición.
   */
  let pendingRun = false;
  let disposed = false;

  function clearTimer(): void {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function schedule(delayMs: number): void {
    if (disposed) {
      return;
    }
    // Defensa: nunca dejar dos temporizadores vivos a la vez.
    clearTimer();
    timer = setTimeout(() => {
      timer = null;
      start();
    }, delayMs);
  }

  function start(): void {
    if (disposed) {
      return;
    }

    // Un solo drenado en vuelo. Si ya hay uno, se marca la intención para
    // forzar un intento inmediato al terminar el que está en curso.
    if (draining) {
      pendingRun = true;
      return;
    }

    draining = true;
    options.drain().then(onSettled, onRejected);
  }

  /**
   * El drenado rechazó (fallo transitorio: p. ej. una excepción inesperada).
   * Se trata como un fallo de red: se reprograma con backoff si quedan
   * pendientes, en lugar de dejar la cola parada.
   */
  function onRejected(): void {
    draining = false;

    if (disposed) {
      return;
    }

    // Cola vacía: no hay nada que reintentar.
    if (!options.hasPending()) {
      attempt = 0;
      pendingRun = false;
      return;
    }

    // Petición coalescida: intento inmediato al terminar este rechazo.
    if (pendingRun) {
      pendingRun = false;
      attempt = 0;
      schedule(0);
      return;
    }

    const delay = backoff[Math.min(attempt, backoff.length - 1)] ?? 0;
    attempt += 1;
    schedule(delay);
  }

  function onSettled(result: DrainResult): void {
    draining = false;

    // 1. Planificador liberado.
    if (disposed) {
      return;
    }

    // 2. Progreso: reinicia el backoff.
    if (result.processed > 0) {
      attempt = 0;
    }

    // 3. Cola vacía: no se programa nada y el backoff queda reiniciado.
    if (!options.hasPending()) {
      attempt = 0;
      pendingRun = false;
      return;
    }

    // 4. `run()` coalescido durante el drenado: intento inmediato.
    if (pendingRun) {
      pendingRun = false;
      attempt = 0;
      schedule(0);
      return;
    }

    // 5. Progreso con pendientes: reintento inmediato.
    if (result.processed > 0) {
      schedule(0);
      return;
    }

    // 6. Fallo de red: espera creciente y acotada al máximo.
    if (result.stoppedBy === 'network') {
      const delay = backoff[Math.min(attempt, backoff.length - 1)] ?? 0;
      attempt += 1;
      schedule(delay);
      return;
    }

    // 7. 'auth', 'error' o null: sin reprogramación; se espera a un nuevo `run`.
  }

  return {
    run(): void {
      if (disposed) {
        return;
      }
      clearTimer();
      attempt = 0;
      start();
    },

    dispose(): void {
      disposed = true;
      clearTimer();
    },
  };
}
