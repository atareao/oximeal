import { useEffect, type ReactNode } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { Spin } from 'antd';
import { redirectToLogin } from './redirect';
import { useSession } from './session';

/** Estado de carga mostrado mientras no se conoce la sesión. */
function AuthLoading() {
  return (
    <div
      data-testid="auth-loading"
      role="status"
      aria-live="polite"
      style={{ display: 'flex', justifyContent: 'center', padding: '48px 0' }}
    >
      <Spin size="large" />
    </div>
  );
}

export interface AuthGuardProps {
  children?: ReactNode;
}

/**
 * Guard de las rutas de datos.
 *
 * - `loading`: muestra un estado de carga y no renderiza los datos.
 * - `anonymous`: redirige a `/auth/login` conservando el destino actual.
 * - `authenticated` / `disabled` (modo abierto) / `degraded` (sin conexión con
 *   datos cacheados): renderiza el contenido.
 */
export function AuthGuard({ children }: AuthGuardProps) {
  const { status } = useSession();
  const location = useLocation();
  const destination = `${location.pathname}${location.search}`;

  useEffect(() => {
    if (status === 'anonymous') {
      redirectToLogin(destination);
    }
  }, [status, destination]);

  if (status === 'loading' || status === 'anonymous') {
    return <AuthLoading />;
  }

  return <>{children ?? <Outlet />}</>;
}
