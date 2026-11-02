# Tasks — Lazy-load de antd (TDD: RED → GREEN → REFACTOR)

## 1. Preparación y medición
- [x] 1.1 Línea base: `pnpm build` y anotar tamaños (antd ~694 kB / ~225 kB gzip)
- [x] 1.2 Confirmar que `main.tsx`, `App.tsx`, `AppLayout`, `AuthGuard`, `ErrorBoundary` y `api/client.ts` importan antd estáticamente

## 2. RED
- [x] 2.1 Añadir `scripts/check-bundle.mjs` que lea el manifest de Vite y falle si el grafo estático inicial supera 500 kB
- [x] 2.2 Ejecutar el script contra el build actual y confirmar que falla (RED)

## 3. GREEN
- [x] 3.1 Crear `AppRoot.tsx` con `ConfigProvider`, `AntdApp`, `ForbiddenNotifier`, `ErrorBoundary`, `BrowserRouter` y `App`
- [x] 3.2 Crear `BootFallback` y `BootErrorBoundary` sin antd
- [x] 3.3 `main.tsx`: lazy import de `AppRoot` + `Suspense` + `BootErrorBoundary`
- [x] 3.4 `api/client.ts`: sustituir el import estático de `message` por `import('antd')` dinámico
- [x] 3.5 Activar `build.manifest` en `vite.config.ts`
- [x] 3.6 `pnpm build` y `node scripts/check-bundle.mjs` en verde (GREEN)

## 4. Tests y cierre
- [x] 4.1 Ajustar `client.test.ts` al aviso de 403 dinámico
- [x] 4.2 `pnpm typecheck`, `pnpm test`, `pnpm lint`, `pnpm build` en verde
- [x] 4.3 Integrar `check-bundle` en `just ci-ui`
- [x] 4.4 `openspec validate antd-lazy-boot --strict` y `openspec validate --all --strict`
- [x] 4.5 Actualizar `PLAN.md` y `CHANGELOG.md`; archivar el change
