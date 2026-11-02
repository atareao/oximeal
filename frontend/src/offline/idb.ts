/**
 * Wrapper tipado de IndexedDB para la caché de consultas.
 *
 * La interfaz {@link OfflineStore} es inyectable para poder testear la
 * persistencia sin depender de `fake-indexeddb`. La implementación real
 * degrada con elegancia cuando `indexedDB` no existe (jsdom, navegadores
 * antiguos): nunca lanza y se comporta como un store vacío.
 */

export interface CachedQuery {
  /** `queryHash` de react-query. */
  key: string;
  /** Estado deshidratado de la consulta. */
  state: unknown;
  updatedAt: number;
}

export interface OfflineStore {
  readQueries(): Promise<CachedQuery[]>;
  writeQueries(entries: CachedQuery[]): Promise<void>;
  /**
   * Sustituye el contenido del store por `entries` en una sola transacción.
   * Devuelve `true` si la transacción se completó correctamente.
   */
  replaceQueries(entries: CachedQuery[]): Promise<boolean>;
  clearQueries(): Promise<void>;
}

export const DB_NAME = 'oximeal';
export const DB_VERSION = 2;
const STORE_NAME = 'queryCache';
const MUTATION_STORE_NAME = 'mutationQueue';

/**
 * Abre (y migra) la base de datos compartida. Resuelve con `null` si IndexedDB
 * no está disponible o la apertura falla.
 *
 * La creación de stores es idempotente: en `onupgradeneeded` se crean **ambos**
 * stores si faltan, de modo que un usuario con la versión 1 (solo
 * `queryCache`) migra a la versión 2 sin perder la caché de consultas.
 *
 * No se ocupa de cachear la conexión: eso lo hace {@link createIdbStore}.
 */
export function openOximealDatabase(): Promise<IDBDatabase | null> {
  if (typeof indexedDB === 'undefined') {
    return Promise.resolve(null);
  }

  return new Promise((resolve) => {
    try {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME, { keyPath: 'key' });
        }
        if (!db.objectStoreNames.contains(MUTATION_STORE_NAME)) {
          db.createObjectStore(MUTATION_STORE_NAME, { keyPath: 'seq', autoIncrement: true });
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => resolve(null);
      // `onblocked` no resuelve: dejamos que `open` siga su curso para no
      // filtrar una conexión abierta si acaba concediéndose.
    } catch {
      resolve(null);
    }
  });
}

export function createIdbStore(): OfflineStore {
  let connection: Promise<IDBDatabase | null> | null = null;

  function getConnection(): Promise<IDBDatabase | null> {
    if (connection) {
      return connection;
    }

    const pending = openOximealDatabase().then((db) => {
      if (db) {
        // Si otra pestaña/versión necesita actualizar la DB, cerramos esta
        // conexión e invalidamos la caché para reabrir con la nueva versión.
        db.onversionchange = () => {
          db.close();
          connection = null;
        };
      } else {
        // No cachear el `null`: permite reintentar en una llamada posterior.
        connection = null;
      }
      return db;
    });

    connection = pending;
    return pending;
  }

  return {
    async readQueries(): Promise<CachedQuery[]> {
      const db = await getConnection();
      if (!db) {
        return [];
      }

      return new Promise((resolve) => {
        try {
          const transaction = db.transaction(STORE_NAME, 'readonly');
          const request = transaction.objectStore(STORE_NAME).getAll();
          request.onsuccess = () => resolve((request.result as CachedQuery[]) ?? []);
          request.onerror = () => resolve([]);
          transaction.onerror = () => resolve([]);
          // Una transacción puede abortar sin emitir `error`; sin esto la
          // promesa nunca resolvería y el arranque se quedaría en blanco.
          transaction.onabort = () => resolve([]);
          transaction.oncomplete = () => resolve([]);
        } catch {
          resolve([]);
        }
      });
    },

    async writeQueries(entries: CachedQuery[]): Promise<void> {
      const db = await getConnection();
      if (!db || entries.length === 0) {
        return;
      }

      await new Promise<void>((resolve) => {
        try {
          const transaction = db.transaction(STORE_NAME, 'readwrite');
          const store = transaction.objectStore(STORE_NAME);
          // Upsert: `put` sobrescribe por `keyPath` en una sola transacción.
          for (const entry of entries) {
            store.put(entry);
          }
          transaction.oncomplete = () => resolve();
          transaction.onerror = () => resolve();
          transaction.onabort = () => resolve();
        } catch {
          resolve();
        }
      });
    },

    async replaceQueries(entries: CachedQuery[]): Promise<boolean> {
      const db = await getConnection();
      if (!db) {
        return false;
      }

      return new Promise<boolean>((resolve) => {
        try {
          const transaction = db.transaction(STORE_NAME, 'readwrite');
          const store = transaction.objectStore(STORE_NAME);
          // Limpia y reescribe en la misma transacción: las claves que ya no
          // están en la caché de react-query desaparecen del store.
          store.clear();
          for (const entry of entries) {
            store.put(entry);
          }
          transaction.oncomplete = () => resolve(true);
          transaction.onerror = () => resolve(false);
          transaction.onabort = () => resolve(false);
        } catch {
          resolve(false);
        }
      });
    },

    async clearQueries(): Promise<void> {
      const db = await getConnection();
      if (!db) {
        return;
      }

      await new Promise<void>((resolve) => {
        try {
          const transaction = db.transaction(STORE_NAME, 'readwrite');
          transaction.objectStore(STORE_NAME).clear();
          transaction.oncomplete = () => resolve();
          transaction.onerror = () => resolve();
          transaction.onabort = () => resolve();
        } catch {
          resolve();
        }
      });
    },
  };
}
