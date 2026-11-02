import { useCallback, useMemo, useSyncExternalStore } from 'react';

function hasMatchMedia(): boolean {
  return typeof window !== 'undefined' && typeof window.matchMedia === 'function';
}

/**
 * Suscribe un media query CSS y devuelve si coincide.
 * Devuelve `false` en entornos sin `matchMedia` (SSR/tests sin mock).
 *
 * El `MediaQueryList` se memoiza por `query` y `subscribe`/`getSnapshot` se
 * estabilizan con `useCallback` para no re-suscribir en cada render.
 */
export function useMediaQuery(query: string): boolean {
  const mediaQueryList = useMemo<MediaQueryList | null>(() => {
    if (!hasMatchMedia()) {
      return null;
    }
    return window.matchMedia(query);
  }, [query]);

  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      if (!mediaQueryList) {
        return () => {};
      }
      mediaQueryList.addEventListener('change', onStoreChange);
      return () => {
        mediaQueryList.removeEventListener('change', onStoreChange);
      };
    },
    [mediaQueryList],
  );

  const getSnapshot = useCallback(() => mediaQueryList?.matches ?? false, [mediaQueryList]);

  const getServerSnapshot = useCallback(() => false, []);

  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}
