/**
 * Provider sin UI que arranca la cola de escrituras offline y la drena cuando
 * hay conexión y operaciones pendientes.
 *
 * Se monta dentro de `QueryClientProvider` y `SessionProvider`, sin envolver el
 * árbol visible.
 */

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { isOnline, useOnline } from './connectivity';
import { getQueuedMutations, initMutationQueue, subscribeMutations } from './mutationQueue';
import { syncShoppingQueue } from './syncShoppingQueue';

export function MutationQueueProvider() {
  const queryClient = useQueryClient();
  const online = useOnline();

  // Carga la cola persistida y, si hay conexión, drena lo que hubiera.
  useEffect(() => {
    let cancelled = false;

    void initMutationQueue().then(() => {
      if (!cancelled && isOnline() && getQueuedMutations().length > 0) {
        void syncShoppingQueue(queryClient);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [queryClient]);

  // Al recuperar la conexión, drena si quedan operaciones.
  useEffect(() => {
    if (!online || getQueuedMutations().length === 0) {
      return;
    }
    void syncShoppingQueue(queryClient);
  }, [online, queryClient]);

  // Cada cambio en la cola intenta drenar (el drenado es reentrante).
  useEffect(() => {
    return subscribeMutations(() => {
      if (isOnline() && getQueuedMutations().length > 0) {
        void syncShoppingQueue(queryClient);
      }
    });
  }, [queryClient]);

  return null;
}
