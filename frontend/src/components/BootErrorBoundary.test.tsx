import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BootErrorBoundary } from './BootErrorBoundary';

function Boom(): never {
  throw new Error('boom');
}

describe('BootErrorBoundary', () => {
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra la pantalla de error con el botón Recargar cuando un hijo lanza', () => {
    render(
      <BootErrorBoundary>
        <Boom />
      </BootErrorBoundary>,
    );

    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /recargar/i })).toBeInTheDocument();
  });

  it('registra el error no controlado mediante componentDidCatch', () => {
    render(
      <BootErrorBoundary>
        <Boom />
      </BootErrorBoundary>,
    );

    expect(console.error).toHaveBeenCalled();
    expect(vi.mocked(console.error).mock.calls.flat()).toContain(
      'Error no controlado en el arranque',
    );
  });

  it('recarga la página al pulsar el botón Recargar', async () => {
    const user = userEvent.setup();
    const reload = vi.fn();
    const location = window.location;

    Object.defineProperty(window, 'location', {
      configurable: true,
      value: { ...location, reload },
    });

    render(
      <BootErrorBoundary>
        <Boom />
      </BootErrorBoundary>,
    );

    await user.click(screen.getByRole('button', { name: /recargar/i }));

    expect(reload).toHaveBeenCalledTimes(1);

    Object.defineProperty(window, 'location', {
      configurable: true,
      value: location,
    });
  });
});
