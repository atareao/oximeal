import { afterEach, describe, expect, it } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { isOnline, useOnline } from './connectivity';

function setNavigatorOnline(value: boolean) {
  Object.defineProperty(navigator, 'onLine', {
    configurable: true,
    get: () => value,
  });
}

describe('connectivity', () => {
  afterEach(() => {
    setNavigatorOnline(true);
  });

  it('isOnline refleja navigator.onLine', () => {
    setNavigatorOnline(false);
    expect(isOnline()).toBe(false);

    setNavigatorOnline(true);
    expect(isOnline()).toBe(true);
  });

  it('useOnline reacciona a los eventos online/offline', () => {
    setNavigatorOnline(true);
    const { result } = renderHook(() => useOnline());

    expect(result.current).toBe(true);

    act(() => {
      setNavigatorOnline(false);
      window.dispatchEvent(new Event('offline'));
    });
    expect(result.current).toBe(false);

    act(() => {
      setNavigatorOnline(true);
      window.dispatchEvent(new Event('online'));
    });
    expect(result.current).toBe(true);
  });
});
