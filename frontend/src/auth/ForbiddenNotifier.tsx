import { useEffect } from 'react';
import { App } from 'antd';
import { setForbiddenHandler } from '../api/client';

/**
 * Registra el aviso de 403 en el cliente usando el `message` del árbol de
 * antd (`App.useApp()`), en lugar del `message` estático.
 *
 * Debe montarse **dentro** de `<AntdApp>` para heredar el contexto de
 * tema/configuración y que los avisos sean coherentes con el resto de la UI.
 * No renderiza nada.
 */
export function ForbiddenNotifier() {
  const { message } = App.useApp();

  useEffect(() => {
    setForbiddenHandler(() => {
      message.warning('Permisos insuficientes');
    });
    return () => setForbiddenHandler(null);
  }, [message]);

  return null;
}
