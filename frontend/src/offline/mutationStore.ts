/**
 * Persistencia de la cola de mutaciones offline.
 *
 * Mismo patrón que `idb.ts`: la interfaz {@link MutationStore} es inyectable
 * para poder testear sin `fake-indexeddb`. La implementación real usa
 * IndexedDB (store `mutationQueue` de la base compartida `oximeal`) y degrada
 * a una cola **en memoria** cuando IndexedDB no está disponible o falla.
 *
 * Nunca lanza: ante cualquier error de IndexedDB se continúa en memoria dentro
 * de la sesión, de modo que la cola sigue funcionando aunque no se persista.
 */

import { openOximealDatabase } from './idb';

export type QueuedMutation =
  | {
      /** Secuencia de encolado (autoincrement). Ausente antes de persistir. */
      seq?: number;
      kind: 'create';
      /** Identificador de cliente (uuid) que hará idempotente el alta. */
      clientId: string;
      /** Datos del alta, sin `client_id` (se añade al enviar). */
      item: { name: string; amount: number; unit: string; category?: string };
      /** Toggle pendiente antes de sincronizar el alta (`null` si no lo hubo). */
      checked: boolean | null;
      /**
       * Cantidad pendiente antes de sincronizar el alta (`null` si no la hubo).
       * Es un merge parcial del alta: se envía como PATCH tras el POST.
       */
      amount: number | null;
      createdAt: number;
    }
  | {
      seq?: number;
      kind: 'update';
      /** Id real del servidor del ítem a modificar. */
      itemId: string;
      patch: { checked?: boolean; amount?: number };
      /** Momento del cambio (ms epoch) para el criterio del último `updated_at`. */
      updatedAt: number;
    };

export interface MutationStore {
  /** Mutaciones ordenadas por `seq`. */
  readMutations(): Promise<QueuedMutation[]>;
  /** Añade una mutación y devuelve su `seq`. */
  addMutation(mutation: QueuedMutation): Promise<number>;
  /** Upsert por `seq`. */
  putMutation(mutation: QueuedMutation): Promise<void>;
  deleteMutation(seq: number): Promise<void>;
  clearMutations(): Promise<void>;
}

const STORE_NAME = 'mutationQueue';

/** Cola en memoria con las mismas garantías que la persistida. */
function createMemoryStore(): MutationStore {
  let nextSeq = 0;
  let mutations: QueuedMutation[] = [];

  return {
    async readMutations(): Promise<QueuedMutation[]> {
      return [...mutations].sort((a, b) => (a.seq ?? 0) - (b.seq ?? 0));
    },

    async addMutation(mutation: QueuedMutation): Promise<number> {
      nextSeq += 1;
      mutations.push({ ...mutation, seq: nextSeq });
      return nextSeq;
    },

    async putMutation(mutation: QueuedMutation): Promise<void> {
      if (mutation.seq === undefined) {
        return;
      }
      const index = mutations.findIndex((entry) => entry.seq === mutation.seq);
      if (index === -1) {
        mutations.push({ ...mutation });
      } else {
        mutations[index] = { ...mutation };
      }
    },

    async deleteMutation(seq: number): Promise<void> {
      mutations = mutations.filter((entry) => entry.seq !== seq);
    },

    async clearMutations(): Promise<void> {
      mutations = [];
    },
  };
}

/** Operaciones sobre IndexedDB. Rechazan ante error para que el híbrido degrade. */
function createIdbStore(): MutationStore {
  async function withStore<T>(
    mode: IDBTransactionMode,
    run: (store: IDBObjectStore) => IDBRequest<T> | void,
  ): Promise<T | undefined> {
    const db = await openOximealDatabase();
    if (!db) {
      throw new Error('IndexedDB no disponible');
    }

    return new Promise<T | undefined>((resolve, reject) => {
      try {
        const transaction = db.transaction(STORE_NAME, mode);
        const store = transaction.objectStore(STORE_NAME);
        const request = run(store);
        let result: T | undefined;

        if (request) {
          request.onsuccess = () => {
            result = request.result as T;
          };
          request.onerror = () => reject(request.error ?? new Error('Error de IndexedDB'));
        }

        transaction.oncomplete = () => resolve(result);
        transaction.onerror = () => reject(transaction.error ?? new Error('Error de IndexedDB'));
        transaction.onabort = () => reject(transaction.error ?? new Error('Transacción abortada'));
      } catch (error) {
        reject(error);
      }
    });
  }

  return {
    async readMutations(): Promise<QueuedMutation[]> {
      const result = await withStore<QueuedMutation[]>('readonly', (store) => store.getAll());
      return (result ?? []).sort((a, b) => (a.seq ?? 0) - (b.seq ?? 0));
    },

    async addMutation(mutation: QueuedMutation): Promise<number> {
      const record: QueuedMutation =
        mutation.kind === 'create'
          ? {
              kind: 'create',
              clientId: mutation.clientId,
              item: mutation.item,
              checked: mutation.checked,
              amount: mutation.amount,
              createdAt: mutation.createdAt,
            }
          : {
              kind: 'update',
              itemId: mutation.itemId,
              patch: mutation.patch,
              updatedAt: mutation.updatedAt,
            };
      const result = await withStore<IDBValidKey>('readwrite', (store) => store.add(record));
      return Number(result);
    },

    async putMutation(mutation: QueuedMutation): Promise<void> {
      if (mutation.seq === undefined) {
        return;
      }
      await withStore('readwrite', (store) => {
        store.put(mutation);
      });
    },

    async deleteMutation(seq: number): Promise<void> {
      await withStore('readwrite', (store) => {
        store.delete(seq);
      });
    },

    async clearMutations(): Promise<void> {
      await withStore('readwrite', (store) => {
        store.clear();
      });
    },
  };
}

/**
 * Crea el store de mutaciones.
 *
 * Usa IndexedDB si existe y, si la apertura o cualquier operación falla,
 * degrada a una cola en memoria durante el resto de la sesión.
 */
export function createMutationStore(): MutationStore {
  if (typeof indexedDB === 'undefined') {
    return createMemoryStore();
  }

  const memory = createMemoryStore();
  const idb = createIdbStore();
  let useMemory = false;

  async function run<T>(idbOp: () => Promise<T>, memoryOp: () => Promise<T>): Promise<T> {
    if (useMemory) {
      return memoryOp();
    }
    try {
      return await idbOp();
    } catch {
      useMemory = true;
      return memoryOp();
    }
  }

  return {
    readMutations: () =>
      run(
        () => idb.readMutations(),
        () => memory.readMutations(),
      ),
    addMutation: (mutation) =>
      run(
        () => idb.addMutation(mutation),
        () => memory.addMutation(mutation),
      ),
    putMutation: (mutation) =>
      run(
        () => idb.putMutation(mutation),
        () => memory.putMutation(mutation),
      ),
    deleteMutation: (seq) =>
      run(
        () => idb.deleteMutation(seq),
        () => memory.deleteMutation(seq),
      ),
    clearMutations: () =>
      run(
        () => idb.clearMutations(),
        () => memory.clearMutations(),
      ),
  };
}
