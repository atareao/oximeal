/**
 * Fachada de la cola de escrituras offline.
 *
 * Mantiene una copia en memoria de las mutaciones encoladas (para que React
 * pueda suscribirse de forma síncrona) y la persiste en {@link MutationStore}.
 * El drenado envía las operaciones al servidor en orden de encolado, con
 * idempotencia en el alta (`client_id`) y last-write-wins por `updated_at`.
 */

import { useSyncExternalStore } from 'react';
import type { ShoppingItem, ShoppingItemInput, ShoppingItemPatch } from '../api/types';
import { createMutationStore, type MutationStore, type QueuedMutation } from './mutationStore';

export const LOCAL_ID_PREFIX = 'local:';

/** Id provisional de un ítem creado sin conexión. */
export function localItemId(clientId: string): string {
  return `${LOCAL_ID_PREFIX}${clientId}`;
}

/** Indica si un id es provisional (creado sin conexión). */
export function isLocalItemId(id: string): boolean {
  return id.startsWith(LOCAL_ID_PREFIX);
}

/** `clientId` a partir de un id provisional. */
export function clientIdFromLocalId(id: string): string {
  return id.slice(LOCAL_ID_PREFIX.length);
}

/** Error de red: `ApiError` con `status === 0` o un `TypeError` de `fetch`. */
export function isNetworkError(error: unknown): boolean {
  if (error instanceof TypeError) {
    return true;
  }
  return (
    typeof error === 'object' && error !== null && (error as { status?: unknown }).status === 0
  );
}

/** Error de autenticación/autorización: 401 o 403. */
export function isAuthError(error: unknown): boolean {
  const status =
    typeof error === 'object' && error !== null
      ? (error as { status?: unknown }).status
      : undefined;
  return status === 401 || status === 403;
}

/** 4xx no recuperable: 400/404/409/422… (excluye 401 y 403). */
export function isNonRecoverableClientError(error: unknown): boolean {
  const status =
    typeof error === 'object' && error !== null
      ? (error as { status?: unknown }).status
      : undefined;
  return (
    typeof status === 'number' && status >= 400 && status < 500 && status !== 401 && status !== 403
  );
}

/** Detecta una marca de zona horaria (`Z` o `±HH:MM`/`±HHMM`) al final. */
const HAS_TIME_ZONE = /(?:Z|[+-]\d{2}:?\d{2})$/;

/**
 * Convierte a epoch en ms un `updated_at` del backend.
 *
 * SQLite devuelve `CURRENT_TIMESTAMP` como `YYYY-MM-DD HH:MM:SS` en UTC pero
 * sin marca de zona; `Date.parse` lo leería como hora local. Acepta también
 * ISO-8601 con `Z` (los ítems optimistas usan `toISOString()`).
 *
 * Devuelve `null` si el texto no es una fecha válida.
 */
export function parseServerTimestamp(value: string): number | null {
  const normalized = HAS_TIME_ZONE.test(value) ? value : `${value.replace(' ', 'T')}Z`;
  const parsed = Date.parse(normalized);
  return Number.isNaN(parsed) ? null : parsed;
}

let store: MutationStore = createMutationStore();
let mutations: QueuedMutation[] = [];
const listeners = new Set<() => void>();
let initPromise: Promise<void> | null = null;
let drainPromise: Promise<DrainResult> | null = null;

function notify(): void {
  for (const listener of listeners) {
    listener();
  }
}

/** Sustituye el store (tests). Reinicia el estado en memoria. */
export function configureMutationQueue(nextStore: MutationStore): void {
  store = nextStore;
  mutations = [];
  initPromise = null;
  drainPromise = null;
  notify();
}

/** Carga la cola persistida en memoria. Idempotente y tolerante a fallos. */
export function initMutationQueue(): Promise<void> {
  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    try {
      mutations = await store.readMutations();
    } catch {
      mutations = [];
    }
    notify();
  })();

  return initPromise;
}

export function getQueuedMutations(): readonly QueuedMutation[] {
  return mutations;
}

export function subscribeMutations(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Hook `useSyncExternalStore` con la cola actual. */
export function useQueuedMutations(): QueuedMutation[] {
  return useSyncExternalStore(
    subscribeMutations,
    getQueuedMutations,
    getQueuedMutations,
  ) as QueuedMutation[];
}

/** Genera un uuid v4 con `crypto.randomUUID` o un fallback manual. */
function generateUuid(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }

  const bytes = new Uint8Array(16);
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    crypto.getRandomValues(bytes);
  } else {
    for (let index = 0; index < bytes.length; index += 1) {
      bytes[index] = Math.floor(Math.random() * 256);
    }
  }
  // Versión 4 y variante RFC 4122.
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

/** Encola un alta y devuelve el `clientId` y el id provisional. */
export async function enqueueCreate(
  item: ShoppingItemInput,
): Promise<{ clientId: string; localId: string }> {
  const clientId = generateUuid();
  const mutation: QueuedMutation = {
    kind: 'create',
    clientId,
    item: {
      name: item.name,
      amount: item.amount,
      unit: item.unit,
      ...(item.category === undefined ? {} : { category: item.category }),
    },
    checked: null,
    amount: null,
    createdAt: Date.now(),
  };

  const seq = await store.addMutation(mutation);
  mutations = [...mutations, { ...mutation, seq }];
  notify();

  return { clientId, localId: localItemId(clientId) };
}

/**
 * Encola un cambio de un ítem.
 *
 * - Si el id es provisional, el cambio se aplica al `create` encolado como un
 *   **merge parcial del alta** (campos `checked` y/o `amount`), en lugar de
 *   crear una operación nueva. `drainQueue` enviará esos campos como PATCH
 *   después del POST.
 * - Si el id es real, se fusiona con un `update` previo del mismo ítem para no
 *   acumular operaciones.
 */
export async function enqueueUpdate(itemId: string, patch: ShoppingItemPatch): Promise<void> {
  if (isLocalItemId(itemId)) {
    const clientId = clientIdFromLocalId(itemId);
    const existing = mutations.find(
      (mutation) => mutation.kind === 'create' && mutation.clientId === clientId,
    );
    if (!existing || existing.kind !== 'create') {
      return;
    }

    const checked = patch.checked === undefined ? existing.checked : patch.checked;
    const amount = patch.amount === undefined ? existing.amount : patch.amount;
    if (checked === existing.checked && amount === existing.amount) {
      return;
    }

    const updated: QueuedMutation = { ...existing, checked, amount };
    await store.putMutation(updated);
    mutations = mutations.map((mutation) => (mutation.seq === existing.seq ? updated : mutation));
    notify();
    return;
  }

  const existing = mutations.find(
    (mutation) => mutation.kind === 'update' && mutation.itemId === itemId,
  );

  if (existing && existing.kind === 'update') {
    const updated: QueuedMutation = {
      ...existing,
      patch: { ...existing.patch, ...patch },
      updatedAt: Date.now(),
    };
    await store.putMutation(updated);
    mutations = mutations.map((mutation) => (mutation.seq === existing.seq ? updated : mutation));
    notify();
    return;
  }

  const mutation: QueuedMutation = {
    kind: 'update',
    itemId,
    patch: { ...patch },
    updatedAt: Date.now(),
  };
  const seq = await store.addMutation(mutation);
  mutations = [...mutations, { ...mutation, seq }];
  notify();
}

export interface DrainOptions {
  createItem(input: ShoppingItemInput): Promise<ShoppingItem>;
  patchItem(id: string, patch: ShoppingItemPatch): Promise<ShoppingItem>;
  /** Ítem creado en el servidor para un alta encolada. */
  onCreated?(clientId: string, item: ShoppingItem): void;
  /** Operación descartada por un 4xx no recuperable. */
  onDiscard?(mutation: QueuedMutation, error: unknown): void;
  /** Estado de la lista en caché, para el criterio del último `updated_at`. */
  getCachedItem?(id: string): ShoppingItem | undefined;
}

export interface DrainResult {
  processed: number;
  discarded: number;
  stoppedBy: 'network' | 'auth' | 'error' | null;
}

async function removeMutation(seq: number): Promise<void> {
  await store.deleteMutation(seq);
  mutations = mutations.filter((mutation) => mutation.seq !== seq);
}

/**
 * Drena la cola en orden de `seq`.
 *
 * - Reentrante: dos llamadas concurrentes comparten la misma promesa.
 * - Se detiene en el primer fallo de red o de autenticación, conservando las
 *   operaciones pendientes.
 * - Un 4xx no recuperable descarta solo esa operación (con `onDiscard`) y
 *   continúa con la siguiente.
 * - En un `update`, si la caché tiene un `updated_at` posterior al momento del
 *   cambio local, gana el servidor: la operación se borra sin avisar (no es un
 *   error, es la resolución normal del conflicto).
 */
export function drainQueue(options: DrainOptions): Promise<DrainResult> {
  if (drainPromise) {
    return drainPromise;
  }

  drainPromise = runDrain(options).finally(() => {
    drainPromise = null;
  });

  return drainPromise;
}

async function runDrain(options: DrainOptions): Promise<DrainResult> {
  let processed = 0;
  let discarded = 0;
  let stoppedBy: DrainResult['stoppedBy'] = null;

  const ordered = [...mutations].sort((a, b) => (a.seq ?? 0) - (b.seq ?? 0));

  for (const mutation of ordered) {
    if (mutation.seq === undefined) {
      continue;
    }

    try {
      if (mutation.kind === 'create') {
        const created = await options.createItem({
          ...mutation.item,
          client_id: mutation.clientId,
        });
        options.onCreated?.(mutation.clientId, created);

        // Merge parcial del alta: se envían los campos modificados sin conexión.
        const pendingPatch: ShoppingItemPatch = {};
        if (mutation.checked != null) {
          pendingPatch.checked = mutation.checked;
        }
        if (mutation.amount != null) {
          pendingPatch.amount = mutation.amount;
        }

        if (pendingPatch.checked !== undefined || pendingPatch.amount !== undefined) {
          try {
            await options.patchItem(created.id, pendingPatch);
          } catch (patchError) {
            if (isNonRecoverableClientError(patchError)) {
              options.onDiscard?.(mutation, patchError);
              discarded += 1;
            } else {
              // Red o auth: se conserva el alta para reintentar (es idempotente).
              throw patchError;
            }
          }
        }

        await removeMutation(mutation.seq);
        processed += 1;
        notify();
        continue;
      }

      const cached = options.getCachedItem?.(mutation.itemId);
      if (cached) {
        const serverTime = parseServerTimestamp(cached.updated_at);
        if (serverTime !== null && serverTime > mutation.updatedAt) {
          // Last-write-wins: el servidor ya tiene un cambio más reciente.
          await removeMutation(mutation.seq);
          notify();
          continue;
        }
      }

      await options.patchItem(mutation.itemId, mutation.patch);
      await removeMutation(mutation.seq);
      processed += 1;
      notify();
    } catch (error) {
      if (isNetworkError(error)) {
        stoppedBy = 'network';
        break;
      }
      if (isAuthError(error)) {
        stoppedBy = 'auth';
        break;
      }
      if (isNonRecoverableClientError(error)) {
        options.onDiscard?.(mutation, error);
        await removeMutation(mutation.seq);
        discarded += 1;
        notify();
        continue;
      }
      stoppedBy = 'error';
      break;
    }
  }

  return { processed, discarded, stoppedBy };
}

/** Vacía la cola persistida y la copia en memoria. */
export async function clearMutationQueue(): Promise<void> {
  await store.clearMutations();
  mutations = [];
  notify();
}
