import { StrictMode, Suspense, lazy } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiError } from './api/client';
import { SessionProvider } from './auth/SessionProvider';
import { BootErrorBoundary } from './components/BootErrorBoundary';
import { BootFallback } from './components/BootFallback';
import { hydrateQueryClient, persistQueryClient } from './offline/queryPersist';
import { registerServiceWorker } from './offline/register-sw';
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

/** Devuelve el elemento `#root` o falla si no existe. */
function requireRootElement(): HTMLElement {
  const element = document.getElementById('root');
  if (!element) {
    throw new Error('No se encontró el elemento #root');
  }
  return element;
}

const rootElement = requireRootElement();

const HYDRATION_TIMEOUT_MS = 2000;

/** Resuelve con `fallback` si `promise` no termina a tiempo. */
function withTimeout<T>(promise: Promise<T>, ms: number, fallback: T): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<T>((resolve) => {
    timer = setTimeout(() => resolve(fallback), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

/**
 * Hidrata la caché desde IndexedDB antes del primer render y arranca la
 * persistencia. La hidratación es best-effort: si IndexedDB no responde en
 * 2 s, se arranca igualmente con la caché vacía.
 */
async function bootstrap() {
  await withTimeout(hydrateQueryClient(queryClient), HYDRATION_TIMEOUT_MS, false);

  // La desuscripción no se guarda: la persistencia vive lo que vive la página.
  persistQueryClient(queryClient);

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

  registerServiceWorker();
}

void bootstrap();
