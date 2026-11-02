import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  ApiError,
  fetchAuthMe,
  logoutAuth,
  setCsrfToken,
  setUnauthorizedHandler,
} from '../api/client';
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

const mockedFetchAuthMe = vi.mocked(fetchAuthMe);
const mockedLogoutAuth = vi.mocked(logoutAuth);
const mockedSetCsrfToken = vi.mocked(setCsrfToken);
const mockedSetUnauthorizedHandler = vi.mocked(setUnauthorizedHandler);

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

function renderProvider() {
  return render(
    <SessionProvider>
      <Probe />
    </SessionProvider>,
  );
}

describe('SessionProvider', () => {
  beforeEach(() => {
    mockedFetchAuthMe.mockReset();
    mockedLogoutAuth.mockReset();
    mockedSetCsrfToken.mockReset();
    mockedSetUnauthorizedHandler.mockReset();
    mockedLogoutAuth.mockResolvedValue(undefined);
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
