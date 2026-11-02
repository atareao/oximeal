import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {
  ApiError,
  fetchAuthMe,
  logoutAuth,
  setCsrfToken,
  setUnauthorizedHandler,
} from '../api/client';
import { isOnline, useOnline } from '../offline/connectivity';
import { clearPersistedQueries, hasCachedData } from '../offline/queryPersist';
import { SessionProvider } from './SessionProvider';
import { useSession } from './session';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchAuthMe: vi.fn(),
    logoutAuth: vi.fn(),
    setCsrfToken: vi.fn(),
    setUnauthorizedHandler: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  isOnline: vi.fn(() => true),
  useOnline: vi.fn(() => true),
}));

vi.mock('../offline/queryPersist', () => ({
  hasCachedData: vi.fn(() => false),
  clearPersistedQueries: vi.fn(async () => undefined),
}));

const mockedFetchAuthMe = vi.mocked(fetchAuthMe);
const mockedLogoutAuth = vi.mocked(logoutAuth);
const mockedSetCsrfToken = vi.mocked(setCsrfToken);
const mockedSetUnauthorizedHandler = vi.mocked(setUnauthorizedHandler);
const mockedIsOnline = vi.mocked(isOnline);
const mockedUseOnline = vi.mocked(useOnline);
const mockedHasCachedData = vi.mocked(hasCachedData);
const mockedClearPersistedQueries = vi.mocked(clearPersistedQueries);

function Probe() {
  const session = useSession();
  return (
    <div>
      <span data-testid="status">{session.status}</span>
      <span data-testid="name">{session.name ?? ''}</span>
      <button type="button" onClick={() => void session.logout().catch(() => undefined)}>
        salir
      </button>
    </div>
  );
}

let queryClient: QueryClient;

function providerTree() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionProvider>
        <Probe />
      </SessionProvider>
    </QueryClientProvider>
  );
}

function renderProvider() {
  queryClient = new QueryClient();
  const result = render(providerTree());
  return {
    ...result,
    queryClient,
    rerenderProvider: () => result.rerender(providerTree()),
  };
}

describe('SessionProvider', () => {
  beforeEach(() => {
    mockedFetchAuthMe.mockReset();
    mockedLogoutAuth.mockReset();
    mockedSetCsrfToken.mockReset();
    mockedSetUnauthorizedHandler.mockReset();
    mockedIsOnline.mockReset();
    mockedUseOnline.mockReset();
    mockedHasCachedData.mockReset();
    mockedClearPersistedQueries.mockReset();

    mockedLogoutAuth.mockResolvedValue(undefined);
    mockedIsOnline.mockReturnValue(true);
    mockedUseOnline.mockReturnValue(true);
    mockedHasCachedData.mockReturnValue(false);
    mockedClearPersistedQueries.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('parte de un estado de carga mientras /auth/me no responde', () => {
    mockedFetchAuthMe.mockReturnValue(new Promise(() => {}));
    renderProvider();

    expect(screen.getByTestId('status')).toHaveTextContent('loading');
    expect(screen.queryByTestId('name')).toHaveTextContent('');
  });

  it('pasa a autenticado y guarda el token CSRF cuando /auth/me responde con sesión', async () => {
    mockedFetchAuthMe.mockResolvedValue({
      authenticated: true,
      subject: 'sub-1',
      name: 'Ana',
      permissions: ['read:recipes'],
      csrf_token: 'csrf-1',
    });
    renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));
    expect(screen.getByTestId('name')).toHaveTextContent('Ana');
    expect(mockedSetCsrfToken).toHaveBeenCalledWith('csrf-1');
  });

  it('queda anónimo cuando /auth/me responde sin autenticar', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: false });
    renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
  });

  it('pasa a modo abierto (disabled) cuando /auth/me devuelve 404', async () => {
    mockedFetchAuthMe.mockRejectedValue(new ApiError(404, 'not_found', 'no existe'));
    renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('disabled'));
  });

  it('vuelve a anónimo cuando el handler global de 401 se dispara', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana' });
    renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    const handler = mockedSetUnauthorizedHandler.mock.calls.at(-1)?.[0];
    expect(handler).toBeTypeOf('function');
    act(() => handler?.());

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
  });

  it('con 401 sin conexión y datos cacheados pasa a degradado conservando la identidad', async () => {
    mockedFetchAuthMe.mockResolvedValue({
      authenticated: true,
      name: 'Ana',
      permissions: ['read:recipes'],
    });
    renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    mockedIsOnline.mockReturnValue(false);
    mockedHasCachedData.mockReturnValue(true);

    const handler = mockedSetUnauthorizedHandler.mock.calls.at(-1)?.[0];
    act(() => handler?.());

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('degraded'));
    expect(screen.getByTestId('name')).toHaveTextContent('Ana');
  });

  it('con 401 y conexión pasa a anónimo aunque haya datos cacheados', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana' });
    const { queryClient: client } = renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    client.setQueryData(['shopping-list'], [{ id: '1' }]);
    mockedIsOnline.mockReturnValue(true);
    mockedHasCachedData.mockReturnValue(true);

    const handler = mockedSetUnauthorizedHandler.mock.calls.at(-1)?.[0];
    act(() => handler?.());

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
    expect(mockedClearPersistedQueries).toHaveBeenCalledTimes(1);
    expect(client.getQueryCache().getAll()).toHaveLength(0);
  });

  it('un fallo de red al arrancar con datos cacheados pasa a degradado', async () => {
    mockedFetchAuthMe.mockRejectedValue(new ApiError(0, 'network_error', 'sin red'));
    mockedHasCachedData.mockReturnValue(true);
    renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('degraded'));
  });

  it('un fallo de red al arrancar sin datos cacheados queda anónimo', async () => {
    mockedFetchAuthMe.mockRejectedValue(new ApiError(0, 'network_error', 'sin red'));
    mockedHasCachedData.mockReturnValue(false);
    renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
  });

  it('al recuperar la conexión desde degradado revalida la sesión', async () => {
    mockedFetchAuthMe.mockRejectedValueOnce(new ApiError(0, 'network_error', 'sin red'));
    mockedHasCachedData.mockReturnValue(true);
    mockedUseOnline.mockReturnValue(false);
    const { rerenderProvider } = renderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('degraded'));

    mockedUseOnline.mockReturnValue(true);
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana' });
    rerenderProvider();

    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));
    expect(screen.getByTestId('name')).toHaveTextContent('Ana');
  });

  it('logout llama a POST /auth/logout, limpia el token y queda anónimo', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana', csrf_token: 'csrf-1' });
    const user = userEvent.setup();
    renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    await user.click(screen.getByRole('button', { name: 'salir' }));

    expect(mockedLogoutAuth).toHaveBeenCalledTimes(1);
    expect(mockedSetCsrfToken).toHaveBeenCalledWith(null);
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
  });

  it('logout limpia la caché de consultas y la persistida', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana' });
    const user = userEvent.setup();
    const { queryClient: client } = renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    client.setQueryData(['shopping-list'], [{ id: '1' }]);
    expect(client.getQueryCache().getAll()).toHaveLength(1);

    await user.click(screen.getByRole('button', { name: 'salir' }));

    expect(mockedClearPersistedQueries).toHaveBeenCalledTimes(1);
    expect(client.getQueryCache().getAll()).toHaveLength(0);
  });

  it('queda anónimo aunque el logout falle (500/red): la rama finally limpia la sesión', async () => {
    mockedFetchAuthMe.mockResolvedValue({ authenticated: true, name: 'Ana', csrf_token: 'csrf-1' });
    mockedLogoutAuth.mockRejectedValue(new ApiError(500, 'http_error', 'boom'));
    const user = userEvent.setup();
    renderProvider();
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('authenticated'));

    await user.click(screen.getByRole('button', { name: 'salir' }));

    expect(mockedSetCsrfToken).toHaveBeenCalledWith(null);
    await waitFor(() => expect(screen.getByTestId('status')).toHaveTextContent('anonymous'));
  });

  it('pasa un AbortSignal a fetchAuthMe y lo cancela al desmontar', () => {
    mockedFetchAuthMe.mockReturnValue(new Promise(() => {}));
    const { unmount } = renderProvider();

    const signal = mockedFetchAuthMe.mock.calls[0]?.[0];
    expect(signal).toBeInstanceOf(AbortSignal);
    expect(signal?.aborted).toBe(false);

    unmount();

    expect(signal?.aborted).toBe(true);
  });
});
