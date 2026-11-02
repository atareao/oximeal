# Tasks — Lazy-load de antd (TDD: RED → GREEN → REFACTOR)

## 1. Preparación y medición
- [ ] 1.1 Línea base: `pnpm build` y anotar tamaños (antd ~694 kB / ~225 kB gzip)
- [ ] 1.2 Confirmar que `main.tsx`, `App.tsx`, `AppLayout`, `AuthGuard`, `ErrorBoundary` y `api/client.ts` importan antd estáticamente

## 2. RED
- [ ] 2.1 Añadir `scripts/check-bundle.mjs` que lea el manifest de Vite y falle si el grafo estático inicial supera 500 kB
- [ ] 2.2 Ejecutar el script contra el build actual y confirmar que falla (RED)

## 3. GREEN
- [ ] 3.1 Crear `AppRoot.tsx` con `ConfigProvider`, `AntdApp`, `ForbiddenNotifier`, `ErrorBoundary`, `BrowserRouter` y `App`
- [ ] 3.2 Crear `BootFallback` y `BootErrorBoundary` sin antd
- [ ] 3.3 `main.tsx`: lazy import de `AppRoot` + `Suspense` + `BootErrorBoundary`
- [ ] 3.4 `api/client.ts`: sustituir el import estático de `message` por `import('antd')` dinámico
- [ ] 3.5 Activar `build.manifest` en `vite.config.ts`
- [ ] 3.6 `pnpm build` y `node scripts/check-bundle.mjs` en verde (GREEN)

## 4. Tests y cierre
- [ ] 4.1 Ajustar `client.test.ts` al aviso de 403 dinámico
- [ ] 4.2 `pnpm typecheck`, `pnpm test`, `pnpm lint`, `pnpm build` en verde
- [ ] 4.3 Integrar `check-bundle` en `just ci-ui`
- [ ] 4.4 `openspec validate antd-lazy-boot --strict` y `openspec validate --all --strict`
- [ ] 4.5 Actualizar `PLAN.md` y `CHANGELOG.md`; archivar el change
