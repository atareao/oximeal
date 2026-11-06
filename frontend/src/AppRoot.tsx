import { App as AntdApp, ConfigProvider } from 'antd';
import { BrowserRouter } from 'react-router-dom';
import { App } from './App';
import { ForbiddenNotifier } from './auth/ForbiddenNotifier';
import { ErrorBoundary } from './components/ErrorBoundary';
import { getTheme } from './components/theme';
import { MutationQueueNotifier } from './offline/MutationQueueNotifier';
import { ThemeProvider, useTheme } from './theme/ThemeProvider';

/**
 * Árbol de la aplicación que depende de antd.
 *
 * Se carga de forma diferida desde `main.tsx` (`React.lazy`) para mantener antd
 * fuera del grafo de módulos estático del entry: el primer pintado usa un
 * fallback ligero y antd se solicita como chunk asíncrono.
 */
function ThemedApp() {
  const { resolvedMode } = useTheme();

  return (
    <ConfigProvider theme={getTheme(resolvedMode)}>
      <AntdApp>
        <ForbiddenNotifier />
        <MutationQueueNotifier />
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </AntdApp>
    </ConfigProvider>
  );
}

export default function AppRoot() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <ThemedApp />
      </ThemeProvider>
    </ErrorBoundary>
  );
}
