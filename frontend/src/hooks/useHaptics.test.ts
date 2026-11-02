import { describe, expect, it, vi, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useHaptics } from './useHaptics';

describe('useHaptics', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('llama a navigator.vibrate cuando está disponible', () => {
    const vibrate = vi.fn().mockReturnValue(true);
    vi.stubGlobal('navigator', { ...navigator, vibrate });

    const { result } = renderHook(() => useHaptics());
    result.current(15);

    expect(vibrate).toHaveBeenCalledWith(15);
  });

  it('vibra con el patrón por defecto (10) cuando no recibe argumentos', () => {
    const vibrate = vi.fn().mockReturnValue(true);
    vi.stubGlobal('navigator', { ...navigator, vibrate });

    const { result } = renderHook(() => useHaptics());
    result.current();

    expect(vibrate).toHaveBeenCalledWith(10);
  });

  it('vibra con el array exacto cuando recibe un patrón en array', () => {
    const vibrate = vi.fn().mockReturnValue(true);
    vi.stubGlobal('navigator', { ...navigator, vibrate });

    const { result } = renderHook(() => useHaptics());
    result.current([10, 50, 10]);

    expect(vibrate).toHaveBeenCalledWith([10, 50, 10]);
  });

  it('no rompe cuando navigator.vibrate no existe', () => {
    vi.stubGlobal('navigator', { ...navigator, vibrate: undefined });

    const { result } = renderHook(() => useHaptics());

    expect(() => result.current()).not.toThrow();
  });
});
