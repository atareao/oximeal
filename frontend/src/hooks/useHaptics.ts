import { useCallback } from 'react';

export type HapticPattern = number | number[];

/**
 * Devuelve una función de feedback táctil.
 *
 * DECISIÓN: se usa la Vibration API (`navigator.vibrate`) en lugar de una
 * librería de haptics. En navegadores sin soporte (p. ej. iOS Safari) la
 * llamada es un no-op. Se descartó `web-haptics` por ser una dependencia
 * inmadura (0.0.6) y añadir un workaround frágil para iOS.
 */
export function useHaptics(): (pattern?: HapticPattern) => void {
  return useCallback((pattern: HapticPattern = 10) => {
    if (typeof navigator !== 'undefined' && typeof navigator.vibrate === 'function') {
      navigator.vibrate(pattern);
    }
  }, []);
}
