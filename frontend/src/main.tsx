import { StrictMode, Suspense, lazy } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiError } from './api/client';
import { SessionProvider } from './auth/SessionProvider';
import { BootErrorBoundary } from './components/BootErrorBoundary';
import { BootFallback } from './components/BootFallback';
import './index.css';

// antd vive en `AppRoot`, que se carga de forma diferida para mantenerlo fuera
// del grafo de módulos estático del entry: el primer pintado usa `BootFallback`.
// eslint-disable-next-line react-refresh/only-export-components -- main.tsx es el entry de la SPA, no un módulo con Fast Refresh.
const AppRoot = lazy(() => import('./AppRoot'));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      // No reintentar errores de cliente (4xx): son deterministas.
      retry: (failureCount, error) => {
        if (error instanceof ApiError && error.status >= 400 && error.status < 500) {
          return false;
        }
        return failureCount < 1;
      },
      refetchOnWindowFocus: false,
    },
  },
});

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('No se encontró el elemento #root');
}

createRoot(rootElement).render(
  <StrictMode>
    <BootErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <SessionProvider>
          <Suspense fallback={<BootFallback />}>
            <AppRoot />
          </Suspense>
        </SessionProvider>
      </QueryClientProvider>
    </BootErrorBoundary>
  </StrictMode>,
);
