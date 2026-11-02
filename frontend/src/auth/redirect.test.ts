import { describe, expect, it, vi, afterEach } from 'vitest';
import { loginUrl, redirectToLogin } from './redirect';

function mockLocationAssign() {
  const assign = vi.fn();
  Object.defineProperty(window, 'location', {
    configurable: true,
    writable: true,
    value: { assign },
  });
  return assign;
}

describe('loginUrl', () => {
  it('devuelve la ruta de login sin return_to cuando no se indica destino', () => {
    expect(loginUrl()).toBe('/auth/login');
  });

  it('sin return_to explícito (cadena vacía) tampoco lo añade', () => {
    expect(loginUrl('')).toBe('/auth/login');
  });

  it('codifica el destino interno en return_to', () => {
    expect(loginUrl('/shopping')).toBe('/auth/login?return_to=%2Fshopping');
  });

  it('codifica caracteres especiales del destino', () => {
    expect(loginUrl('/shopping?q=a b&page=1')).toBe(
      '/auth/login?return_to=%2Fshopping%3Fq%3Da%20b%26page%3D1',
    );
  });
});

describe('redirectToLogin', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('navega a /auth/login con return_to cuando se indica destino', () => {
    const assign = mockLocationAssign();

    redirectToLogin('/shopping');

    expect(assign).toHaveBeenCalledTimes(1);
    expect(assign).toHaveBeenCalledWith('/auth/login?return_to=%2Fshopping');
  });

  it('navega a /auth/login sin return_to cuando no se indica destino', () => {
    const assign = mockLocationAssign();

    redirectToLogin();

    expect(assign).toHaveBeenCalledTimes(1);
    expect(assign).toHaveBeenCalledWith('/auth/login');
  });
});
