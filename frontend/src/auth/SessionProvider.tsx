import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
} from 'react';
import { useQueryClient } from '@tanstack/react-query';
import {
  ApiError,
  fetchAuthMe,
  logoutAuth,
  setCsrfToken,
  setUnauthorizedHandler,
} from '../api/client';
import { isOnline, useOnline } from '../offline/connectivity';
import { clearPersistedQueries, hasCachedData } from '../offline/queryPersist';
import { SessionContext, type SessionContextValue, type SessionStatus } from './session';

interface SessionState {
  status: SessionStatus;
  subject?: string;
  name?: string;
  permissions: string[];
}

const INITIAL_STATE: SessionState = { status: 'loading', permissions: [] };
const ANONYMOUS_STATE: SessionState = { status: 'anonymous', permissions: [] };

/** Una petición cancelada vía `AbortController` no debe alterar el estado. */
function isAbortError(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    (error as { name?: unknown }).name === 'AbortError'
  );
}

/**
 * Consulta `GET /auth/me` y aplica el resultado al estado de sesión.
 *
 * - 404: OIDC desactivado → modo abierto (`disabled`).
 * - Fallo de red (`status === 0`) con datos cacheados → `degraded`.
 * - Cualquier otro fallo → `anonymous`.
 */
async function loadSession(
  signal: AbortSignal,
  setState: Dispatch<SetStateAction<SessionState>>,
): Promise<void> {
  try {
    const me = await fetchAuthMe(signal);
    if (signal.aborted) {
      return;
    }
    if (me.authenticated) {
      setCsrfToken(me.csrf_token ?? null);
      setState({
        status: 'authenticated',
        subject: me.subject,
        name: me.name,
        permissions: me.permissions ?? [],
      });
    } else {
      setCsrfToken(null);
      setState(ANONYMOUS_STATE);
    }
  } catch (error) {
    if (signal.aborted || isAbortError(error)) {
      return;
    }
    if (error instanceof ApiError && error.status === 404) {
      setState({ status: 'disabled', permissions: [] });
    } else if (error instanceof ApiError && error.status === 0 && hasCachedData()) {
      setState((prev) => ({ ...prev, status: 'degraded' }));
    } else {
      setState(ANONYMOUS_STATE);
    }
  }
}

/**
 * Provee el estado de sesión consultando `GET /auth/me` al arrancar.
 *
 * - Registra el handler global de 401: sin conexión y con datos cacheados pasa
 *   a `degraded`; en caso contrario, a anónimo (re-login).
 * - Rellena el token CSRF del cliente cuando hay sesión.
 * - Si `/auth/me` responde 404 (OIDC desactivado) la app queda en modo abierto.
 * - Al recuperar la conexión desde `degraded`, revalida la sesión.
 * - Cancela la consulta en vuelo (`AbortController`) al desmontar.
 */
export function SessionProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<SessionState>(INITIAL_STATE);
  const online = useOnline();
  const queryClient = useQueryClient();

  useEffect(() => {
    setUnauthorizedHandler(() => {
      setCsrfToken(null);
      if (!isOnline() && hasCachedData()) {
        setState((prev) => ({ ...prev, status: 'degraded' }));
        return;
      }
      // Sin datos que mostrar: se vuelve al login y se borra la caché para no
      // exponer datos de un usuario a otro en el mismo navegador.
      queryClient.clear();
      void clearPersistedQueries();
      setState(ANONYMOUS_STATE);
    });
    return () => setUnauthorizedHandler(null);
  }, [queryClient]);

  useEffect(() => {
    const controller = new AbortController();
    void loadSession(controller.signal, setState);
    return () => {
      controller.abort();
    };
  }, []);

  // Recuperación de la conexión: revalida la sesión si estábamos degradados.
  useEffect(() => {
    if (!online || state.status !== 'degraded') {
      return;
    }
    const controller = new AbortController();
    void loadSession(controller.signal, setState);
    return () => {
      controller.abort();
    };
  }, [online, state.status]);

  const logout = useCallback(async () => {
    try {
      await logoutAuth();
    } finally {
      setCsrfToken(null);
      queryClient.clear();
      await clearPersistedQueries();
      setState(ANONYMOUS_STATE);
    }
  }, [queryClient]);

  const value = useMemo<SessionContextValue>(() => ({ ...state, logout }), [state, logout]);

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}
