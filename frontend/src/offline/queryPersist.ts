import {
  dehydrate,
  hydrate,
  type DehydratedState,
  type Query,
  type QueryClient,
} from '@tanstack/react-query';
import { createIdbStore, type CachedQuery, type OfflineStore } from './idb';

/** Raíces de `queryKey` cuyos datos se cachean en IndexedDB. */
export const PERSISTED_QUERY_ROOTS = ['shopping-list', 'meal-plan', 'recipes', 'settings'] as const;

const PERSISTED_ROOTS: ReadonlySet<string> = new Set(PERSISTED_QUERY_ROOTS);

const PERSIST_DEBOUNCE_MS = 250;

/**
 * Flag síncrono de disponibilidad de datos cacheados.
 *
 * El handler global de 401 no puede esperar a IndexedDB, así que consulta este
 * flag para decidir entre modo degradado y re-login.
 */
let cachedDataFlag = false;

export function isPersistedQueryKey(queryKey: readonly unknown[]): boolean {
  const root = queryKey[0];
  return typeof root === 'string' && PERSISTED_ROOTS.has(root);
}

/**
 * Solo se persisten consultas con datos: se excluyen las que están en
 * `pending`/`error` sin datos.
 */
function shouldDehydrateQuery(query: Query): boolean {
  if (!isPersistedQueryKey(query.queryKey)) {
    return false;
  }
  if (query.state.status === 'pending' || query.state.status === 'error') {
    return query.state.data !== undefined;
  }
  return true;
}

/** Recupera la `queryKey` de una entrada persistida (defensa en profundidad). */
function entryQueryKey(entry: CachedQuery): readonly unknown[] | null {
  const state = entry.state as { queryKey?: unknown } | null;
  if (state && Array.isArray(state.queryKey)) {
    return state.queryKey;
  }
  try {
    const parsed: unknown = JSON.parse(entry.key);
    return Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

/**
 * Hidrata la caché de react-query desde IndexedDB.
 *
 * @returns `true` si se hidrató al menos una consulta.
 */
export async function hydrateQueryClient(
  client: QueryClient,
  store: OfflineStore = createIdbStore(),
): Promise<boolean> {
  try {
    const entries = await store.readQueries();
    // Defensa en profundidad: nunca hidratar claves fuera de la lista blanca.
    const persisted = entries.filter((entry) => {
      const queryKey = entryQueryKey(entry);
      return queryKey !== null && isPersistedQueryKey(queryKey);
    });

    if (persisted.length === 0) {
      cachedDataFlag = false;
      return false;
    }

    hydrate(client, {
      queries: persisted.map((entry) => entry.state) as DehydratedState['queries'],
    });
    cachedDataFlag = true;
    return true;
  } catch {
    cachedDataFlag = false;
    return false;
  }
}

/**
 * Persiste en IndexedDB las consultas de la lista blanca.
 *
 * - Se suscribe a la caché de consultas con un debounce de 250 ms.
 * - Sustituye el contenido del store en cada escritura, de modo que las claves
 *   que ya no están en la caché se eliminan.
 * - Fuerza la escritura al ocultar la pestaña (`visibilitychange`) y en
 *   `pagehide`, para no perder datos al cerrar.
 *
 * @returns Función de desuscripción.
 */
export function persistQueryClient(
  client: QueryClient,
  store: OfflineStore = createIdbStore(),
): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;

  const persist = () => {
    const state = dehydrate(client, { shouldDehydrateQuery });
    const entries: CachedQuery[] = state.queries.map((query) => ({
      key: query.queryHash,
      state: query,
      updatedAt: Date.now(),
    }));

    if (entries.length === 0) {
      cachedDataFlag = false;
    }

    void store
      .replaceQueries(entries)
      .then((ok) => {
        // El flag refleja el resultado real: solo hay datos si la escritura
        // se completó y la caché no quedó vacía.
        cachedDataFlag = ok && entries.length > 0;
      })
      .catch(() => {
        cachedDataFlag = false;
      });
  };

  const schedule = () => {
    if (timer !== null) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      timer = null;
      persist();
    }, PERSIST_DEBOUNCE_MS);
  };

  const unsubscribe = client.getQueryCache().subscribe(schedule);

  const onVisibilityChange = () => {
    if (document.visibilityState === 'hidden') {
      persist();
    }
  };
  const onPageHide = () => persist();

  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', onVisibilityChange);
  }
  if (typeof window !== 'undefined') {
    window.addEventListener('pagehide', onPageHide);
  }

  return () => {
    unsubscribe();
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', onVisibilityChange);
    }
    if (typeof window !== 'undefined') {
      window.removeEventListener('pagehide', onPageHide);
    }
  };
}

/** Vacía la caché persistida y desactiva el flag de datos disponibles. */
export async function clearPersistedQueries(store: OfflineStore = createIdbStore()): Promise<void> {
  cachedDataFlag = false;
  try {
    await store.clearQueries();
  } catch {
    // La limpieza es best-effort: nunca debe romper el cierre de sesión.
  }
}

/** Indica si hay datos cacheados disponibles (lectura síncrona). */
export function hasCachedData(): boolean {
  return cachedDataFlag;
}
