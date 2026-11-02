# Change: Cargar antd fuera del critical path de arranque

## Why
El code-splitting por rutas bajó el entry a ~11 kB, pero el chunk `antd` sigue siendo un único bloque de ~694 kB (~225 kB gzip) que se carga en el arranque porque el shell (`main.tsx`, `App.tsx`, `AppLayout`, `AuthGuard`, `ErrorBoundary`) y `api/client.ts` importan antd de forma estática. En una app mobile-first la primera carga es cara (issue #13).

## What Changes
- Mover los proveedores de antd (`ConfigProvider`, `AntdApp`), `ForbiddenNotifier`, `ErrorBoundary` y el árbol de rutas a un módulo `AppRoot` cargado con `React.lazy`.
- Dejar `main.tsx` con un fallback ligero en HTML/CSS y un error boundary de arranque sin antd.
- Eliminar el import estático de `message` en `api/client.ts`; el aviso de 403 usará `import('antd')` dinámico.
- Medir el bundle antes/después y añadir una comprobación de presupuesto del grafo estático inicial (< 500 kB en bruto).

## Impact
- Capability: `ui-shell` (requisito añadido).
- Ficheros: `frontend/src/main.tsx`, `frontend/src/AppRoot.tsx` (nuevo), `frontend/src/components/BootFallback.tsx` (nuevo), `frontend/src/components/BootErrorBoundary.tsx` (nuevo), `frontend/src/api/client.ts`, `frontend/vite.config.ts`, `frontend/scripts/check-bundle.mjs` (nuevo), `justfile`, `frontend/package.json`.
- Sin cambios de comportamiento de producto; antd se sigue usando en las páginas.
