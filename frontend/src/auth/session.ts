import { createContext, useContext } from 'react';

/**
 * Estado de sesión del frontend.
 *
 * - `loading`: todavía no se conoce el estado (`GET /auth/me` en vuelo).
 * - `authenticated`: hay sesión con identidad.
 * - `anonymous`: no hay sesión; hay que ir al login.
 * - `disabled`: el backend no monta `/auth/*` (OIDC desactivado); modo abierto.
 * - `degraded`: sin conexión y con datos cacheados; se muestran los datos
 *   guardados en lugar de redirigir al login.
 */
export type SessionStatus = 'loading' | 'authenticated' | 'anonymous' | 'disabled' | 'degraded';

export interface SessionContextValue {
  status: SessionStatus;
  subject?: string;
  name?: string;
  permissions: string[];
  /** Cierra la sesión en el backend, limpia el token CSRF y queda anónimo. */
  logout: () => Promise<void>;
}

export const SessionContext = createContext<SessionContextValue | null>(null);

export function useSession(): SessionContextValue {
  const context = useContext(SessionContext);
  if (!context) {
    throw new Error('useSession debe usarse dentro de <SessionProvider>');
  }
  return context;
}
