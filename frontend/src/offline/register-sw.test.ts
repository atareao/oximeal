import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { registerServiceWorker } from './register-sw';

const originalServiceWorker = Object.getOwnPropertyDescriptor(navigator, 'serviceWorker');

function setReadyState(state: DocumentReadyState): void {
  Object.defineProperty(document, 'readyState', { value: state, configurable: true });
}

describe('registerServiceWorker', () => {
  const register = vi.fn<() => Promise<ServiceWorkerRegistration>>();

  beforeEach(() => {
    register.mockResolvedValue({} as ServiceWorkerRegistration);
    Object.defineProperty(navigator, 'serviceWorker', {
      value: { register },
      configurable: true,
    });
  });

  afterEach(() => {
    vi.unstubAllEnvs();
    register.mockReset();
    if (originalServiceWorker) {
      Object.defineProperty(navigator, 'serviceWorker', originalServiceWorker);
    } else {
      Reflect.deleteProperty(navigator, 'serviceWorker');
    }
    Reflect.deleteProperty(document, 'readyState');
  });

  it('no registra el Service Worker fuera de producción', () => {
    vi.stubEnv('PROD', false);
    setReadyState('complete');

    registerServiceWorker();

    expect(register).not.toHaveBeenCalled();
  });

  it('registra /sw.js con scope / en producción al cargar la ventana', () => {
    vi.stubEnv('PROD', true);
    setReadyState('loading');

    registerServiceWorker();
    expect(register).not.toHaveBeenCalled();

    window.dispatchEvent(new Event('load'));

    expect(register).toHaveBeenCalledWith('/sw.js', { scope: '/' });
  });

  it('registra de inmediato si el documento ya está cargado', () => {
    vi.stubEnv('PROD', true);
    setReadyState('complete');

    registerServiceWorker();

    expect(register).toHaveBeenCalledWith('/sw.js', { scope: '/' });
  });

  it('no propaga el rechazo del registro', async () => {
    vi.stubEnv('PROD', true);
    setReadyState('complete');
    register.mockRejectedValueOnce(new Error('fallo de registro'));
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});

    expect(() => registerServiceWorker()).not.toThrow();
    await Promise.resolve();
    await Promise.resolve();

    expect(warn).toHaveBeenCalled();
  });
});
