/**
 * Drenado de la cola de escrituras offline conectado a la API y a la caché de
 * react-query.
 *
 * Vive en su propio módulo (y no en `hooks.ts`) para evitar ciclos: `hooks.ts`
 * importa la cola, pero no este sincronizador. El provider solo lo monta.
 */

import type { QueryClient } from '@tanstack/react-query';
import { createShoppingItem, patchShoppingItem } from '../api/client';
import { findShoppingItem, queryKeys } from '../api/hooks';
import type { ShoppingItem, ShoppingListGroup } from '../api/types';
import { isOnline } from './connectivity';
import { drainQueue, getQueuedMutations, localItemId, type DrainResult } from './mutationQueue';

type DiscardHandler = (error: unknown) => void;

/**
 * Handler global invocado al descartar una operación por un 4xx no recuperable.
 *
 * Lo registra un componente montado **dentro** de `<AntdApp>` (p. ej.
 * `MutationQueueNotifier`) que usa `App.useApp().message`, de forma que el
 * aviso comparta el contexto de tema/configuración de la aplicación. Sin
 * handler registrado, el descarte se aplica en silencio (ya está resuelto).
 */
let discardHandler: DiscardHandler | null = null;

export function setDiscardHandler(handler: DiscardHandler | null): void {
  discardHandler = handler;
}

function notifyDiscard(error: unknown): void {
  discardHandler?.(error);
}

/** Drenado en curso, para no lanzar uno nuevo mientras otro está activo. */
let syncPromise: Promise<DrainResult> | null = null;

/**
 * Drena la cola y reconcilia la caché de la lista de la compra.
 *
 * - `onCreated` sustituye el ítem provisional por el del servidor (sin
 *   `pending`); si ya no está en la caché, invalida.
 * - `onDiscard` avisa al usuario de un 4xx no recuperable.
 * - `getCachedItem` alimenta el criterio last-write-wins por `updated_at`.
 * - Al terminar con operaciones aplicadas y conexión, invalida la lista.
 *
 * Reentrante: si ya hay un drenado en curso comparte la misma promesa; con la
 * cola vacía no arranca ningún trabajo.
 */
export function syncShoppingQueue(queryClient: QueryClient): Promise<DrainResult> {
  if (syncPromise) {
    return syncPromise;
  }
  if (getQueuedMutations().length === 0) {
    return Promise.resolve({ processed: 0, discarded: 0, stoppedBy: null });
  }

  syncPromise = runSync(queryClient).finally(() => {
    syncPromise = null;
  });

  return syncPromise;
}

async function runSync(queryClient: QueryClient): Promise<DrainResult> {
  const result = await drainQueue({
    createItem: createShoppingItem,
    patchItem: patchShoppingItem,
    onCreated(clientId, item) {
      const localId = localItemId(clientId);
      let replaced = false;

      queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) => {
        if (!old) {
          return old;
        }
        return old.map((group) => ({
          ...group,
          items: group.items.map((entry) => {
            if (entry.id === localId) {
              replaced = true;
              return item;
            }
            return entry;
          }),
        }));
      });

      if (!replaced) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
      }
    },
    onDiscard(_mutation, error) {
      notifyDiscard(error);
    },
    getCachedItem(id): ShoppingItem | undefined {
      return findShoppingItem(
        queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all),
        id,
      );
    },
  });

  if (result.processed > 0 && isOnline()) {
    void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
  }

  return result;
}
