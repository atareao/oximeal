import { App as AntdApp, ConfigProvider } from 'antd';
import { BrowserRouter } from 'react-router-dom';
import { App } from './App';
import { ForbiddenNotifier } from './auth/ForbiddenNotifier';
import { ErrorBoundary } from './components/ErrorBoundary';
import { theme } from './components/theme';
import { MutationQueueNotifier } from './offline/MutationQueueNotifier';

/**
 * Árbol de la aplicación que depende de antd.
 *
 * Se carga de forma diferida desde `main.tsx` (`React.lazy`) para mantener antd
 * fuera del grafo de módulos estático del entry: el primer pintado usa un
 * fallback ligero y antd se solicita como chunk asíncrono.
 */
export default function AppRoot() {
  return (
    <ErrorBoundary>
      <ConfigProvider theme={theme}>
        <AntdApp>
          <ForbiddenNotifier />
          <MutationQueueNotifier />
          <BrowserRouter>
            <App />
          </BrowserRouter>
        </AntdApp>
      </ConfigProvider>
    </ErrorBoundary>
  );
}
