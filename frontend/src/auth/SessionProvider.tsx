import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';
import {
  ApiError,
  fetchAuthMe,
  logoutAuth,
  setCsrfToken,
  setUnauthorizedHandler,
} from '../api/client';
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
 * Provee el estado de sesión consultando `GET /auth/me` al arrancar.
 *
 * - Registra el handler global de 401 para volver a anónimo (re-login).
 * - Rellena el token CSRF del cliente cuando hay sesión.
 * - Si `/auth/me` responde 404 (OIDC desactivado) la app queda en modo abierto.
 * - Cancela la consulta en vuelo (`AbortController`) al desmontar.
 */
export function SessionProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<SessionState>(INITIAL_STATE);

  useEffect(() => {
    setUnauthorizedHandler(() => {
      setCsrfToken(null);
      setState(ANONYMOUS_STATE);
    });
    return () => setUnauthorizedHandler(null);
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    const { signal } = controller;

    fetchAuthMe(signal)
      .then((me) => {
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
      })
      .catch((error: unknown) => {
        if (signal.aborted || isAbortError(error)) {
          return;
        }
        if (error instanceof ApiError && error.status === 404) {
          setState({ status: 'disabled', permissions: [] });
        } else {
          setState(ANONYMOUS_STATE);
        }
      });

    return () => {
      controller.abort();
    };
  }, []);

  const logout = useCallback(async () => {
    try {
      await logoutAuth();
    } finally {
      setCsrfToken(null);
      setState(ANONYMOUS_STATE);
    }
  }, []);

  const value = useMemo<SessionContextValue>(() => ({ ...state, logout }), [state, logout]);

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}
