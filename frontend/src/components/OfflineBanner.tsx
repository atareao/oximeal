import { Alert } from 'antd';
import { useSession } from '../auth/session';
import { useOnline } from '../offline/connectivity';

/**
 * Aviso persistente de modo degradado.
 *
 * - Sin conexión: se muestran los datos guardados.
 * - Sesión caducada sin conexión (`degraded`): se muestran los datos guardados.
 *
 * No renderiza nada con conexión y sesión válida.
 */
export function OfflineBanner() {
  const online = useOnline();
  const { status } = useSession();

  if (online && status !== 'degraded') {
    return null;
  }

  const message = online
    ? 'Sesión caducada sin conexión. Se muestran los datos guardados.'
    : 'Sin conexión. Se muestran los datos guardados.';

  return <Alert data-testid="offline-banner" type="warning" banner showIcon message={message} />;
}
