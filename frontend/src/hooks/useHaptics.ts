import { useCallback } from 'react';

export type HapticPattern = number | number[];

/**
 * Devuelve una función de feedback táctil.
 * Usa `navigator.vibrate` si existe; en caso contrario es un no-op.
 */
export function useHaptics(): (pattern?: HapticPattern) => void {
  return useCallback((pattern: HapticPattern = 10) => {
    if (typeof navigator !== 'undefined' && typeof navigator.vibrate === 'function') {
      navigator.vibrate(pattern);
    }
  }, []);
}
