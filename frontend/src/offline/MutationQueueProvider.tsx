/**
 * Provider sin UI que arranca la cola de escrituras offline y la drena cuando
 * hay conexión y operaciones pendientes.
 *
 * Se monta dentro de `QueryClientProvider` y `SessionProvider`, sin envolver el
 * árbol visible. El drenado lo orquesta {@link createDrainScheduler}, que
 * reintenta solo (con backoff acotado) cuando el fallo es de red y se detiene
 * ante un 401/403 hasta que cambien las condiciones.
 *
 * El planificador pertenece al ciclo de vida del montaje: se crea dentro del
 * efecto dueño y se destruye en su limpieza. Esto es imprescindible bajo
 * `<StrictMode>`, que monta-desmonta-monta los efectos: si el planificador se
 * compartiera entre montajes (p. ej. vía `useMemo`), el `dispose()` del primer
 * desmontaje lo dejaría inutilizado para siempre y `run()` sería un no-op.
 */

import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useOnline } from './connectivity';
import { createDrainScheduler, type DrainScheduler } from './drainScheduler';
import { getQueuedMutations, initMutationQueue, subscribeMutations } from './mutationQueue';
import { syncShoppingQueue } from './syncShoppingQueue';

export function MutationQueueProvider() {
  const queryClient = useQueryClient();
  const online = useOnline();
  const schedulerRef = useRef<DrainScheduler | null>(null);

  // Efecto dueño: crea el planificador, carga la cola y lo libera al desmontar.
  // Debe declararse ANTES que los demás efectos para que `schedulerRef.current`
  // esté listo cuando estos corran (React ejecuta los efectos en orden).
  useEffect(() => {
    const scheduler = createDrainScheduler({
      drain: () => syncShoppingQueue(queryClient),
      hasPending: () => getQueuedMutations().length > 0,
    });
    schedulerRef.current = scheduler;

    let cancelled = false;
    void initMutationQueue().then(() => {
      if (!cancelled) {
        scheduler.run();
      }
    });

    return () => {
      cancelled = true;
      scheduler.dispose();
      schedulerRef.current = null;
    };
  }, [queryClient]);

  // Al recuperar la conexión, intenta drenar.
  useEffect(() => {
    if (online) {
      schedulerRef.current?.run();
    }
  }, [online]);

  // Cada cambio en la cola intenta drenar (`run` es reentrante y barato).
  useEffect(() => {
    return subscribeMutations(() => {
      schedulerRef.current?.run();
    });
  }, []);

  // Al volver al primer plano o recuperar el foco, reintenta de inmediato.
  useEffect(() => {
    const onVisibilityChange = (): void => {
      if (document.visibilityState === 'visible') {
        schedulerRef.current?.run();
      }
    };
    const onFocus = (): void => {
      schedulerRef.current?.run();
    };

    document.addEventListener('visibilitychange', onVisibilityChange);
    window.addEventListener('focus', onFocus);

    return () => {
      document.removeEventListener('visibilitychange', onVisibilityChange);
      window.removeEventListener('focus', onFocus);
    };
  }, []);

  return null;
}
