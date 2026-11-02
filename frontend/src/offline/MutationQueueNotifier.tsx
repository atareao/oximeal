import { useEffect } from 'react';
import { App } from 'antd';
import { setDiscardHandler } from './syncShoppingQueue';

/**
 * Registra el aviso de operación descartada usando el `message` del árbol de
 * antd (`App.useApp()`), en lugar del `message` estático.
 *
 * Debe montarse **dentro** de `<AntdApp>` para heredar el contexto de
 * tema/configuración y que los avisos sean coherentes con el resto de la UI.
 * No renderiza nada.
 */
export function MutationQueueNotifier() {
  const { message } = App.useApp();

  useEffect(() => {
    setDiscardHandler((error) => {
      const detail = error instanceof Error ? error.message : 'error desconocido';
      message.error(`Se descartó un cambio pendiente: ${detail}`);
    });
    return () => setDiscardHandler(null);
  }, [message]);

  return null;
}
