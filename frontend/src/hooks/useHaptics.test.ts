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

  it('no rompe cuando navigator.vibrate no existe', () => {
    vi.stubGlobal('navigator', { ...navigator, vibrate: undefined });

    const { result } = renderHook(() => useHaptics());

    expect(() => result.current()).not.toThrow();
  });
});
