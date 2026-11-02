import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { message } from 'antd';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { apiRequest } from '../api/client';
import { AuthGuard } from './AuthGuard';
import { SessionProvider } from './SessionProvider';
import { useSession } from './session';

/**
 * Test de integración con el cliente real (sin mocks de `../api/client`):
 * verifica que un 401 dispara el re-login a través de SessionProvider→AuthGuard
 * y que un 403 mantiene la sesión intacta a nivel de contexto.
 */

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function installFetch() {
  const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
    const url = typeof input === 'string' ? input : input.toString();
    if (url === '/auth/me') {
      return jsonResponse({
        authenticated: true,
        name: 'Ana',
        permissions: ['read:recipes'],
        csrf_token: 'csrf-1',
      });
    }
    if (url === '/api/recipes') {
      return jsonResponse({ error: 'forbidden', message: 'permiso denegado' }, 403);
    }
    if (url === '/api/shopping-list') {
      return jsonResponse({ error: 'unauthorized', message: 'sesión caducada' }, 401);
    }
    return jsonResponse({}, 200);
  });
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

function mockLocationAssign() {
  const assign = vi.fn();
  Object.defineProperty(window, 'location', {
    configurable: true,
    writable: true,
    value: { assign },
  });
  return assign;
}

function Probe() {
  const { status, logout } = useSession();
  return (
    <div>
      <span data-testid="status">{status}</span>
      <button type="button" onClick={() => void apiRequest('/recipes').catch(() => undefined)}>
        403
      </button>
      <button
        type="button"
        onClick={() => void apiRequest('/shopping-list').catch(() => undefined)}
      >
        401
      </button>
      <button type="button" onClick={() => void logout().catch(() => undefined)}>
        salir
      </button>
    </div>
  );
}

function renderSession() {
  const queryClient = new QueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/shopping']}>
        <SessionProvider>
          <Probe />
          <AuthGuard>
            <div data-testid="protected">datos</div>
          </AuthGuard>
        </SessionProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe('SessionProvider + AuthGuard (integración 401/403)', () => {
  beforeEach(() => {
    installFetch();
    mockLocationAssign();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('un 403 muestra aviso pero mantiene la sesión autenticada y no redirige', async () => {
    const assign = mockLocationAssign();
    const warning = vi.spyOn(message, 'warning').mockImplementation((() => {}) as never);
    const user = userEvent.setup();
    renderSession();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    await user.click(screen.getByRole('button', { name: '403' }));

    await waitFor(() => expect(warning).toHaveBeenCalled());
    expect(screen.getByTestId('status')).toHaveTextContent('authenticated');
    expect(assign).not.toHaveBeenCalled();
    expect(screen.getByTestId('protected')).toBeInTheDocument();
  });

  it('un 401 deja la sesión anónima y AuthGuard vuelve al login con el destino', async () => {
    const assign = mockLocationAssign();
    const user = userEvent.setup();
    renderSession();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    await user.click(screen.getByRole('button', { name: '401' }));

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
    await waitFor(() => expect(assign).toHaveBeenCalledWith('/auth/login?return_to=%2Fshopping'));
    expect(screen.queryByTestId('protected')).not.toBeInTheDocument();
  });

  it('tras un logout correcto AuthGuard vuelve al login (sin doble navegación)', async () => {
    const assign = mockLocationAssign();
    const user = userEvent.setup();
    renderSession();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    await user.click(screen.getByRole('button', { name: 'salir' }));

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
    await waitFor(() => expect(assign).toHaveBeenCalledWith('/auth/login?return_to=%2Fshopping'));
    expect(assign).toHaveBeenCalledTimes(1);
  });
});
