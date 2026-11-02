# Tasks — Caché de datos offline y modo degradado (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Revisar `queryKeys` en `frontend/src/api/hooks.ts` y el `QueryClient` de `main.tsx`
- [x] 1.2 Revisar el manejo de 401 en `client.ts`, `SessionProvider.tsx` y `AuthGuard.tsx`

## 2. RED
- [x] 2.1 Test: hidratar la caché desde IndexedDB al arrancar
- [x] 2.2 Test: persistir el resultado de una consulta correcta
- [x] 2.3 Test: con datos cacheados y sin red, la vista muestra los datos y no redirige al login
- [x] 2.4 Test: con red y 401, la aplicación redirige al login
- [x] 2.5 Test: el cierre de sesión limpia la caché de datos de IndexedDB
- [x] 2.6 Ejecutar `pnpm test`; confirmar el estado RED

## 3. GREEN
- [x] 3.1 `frontend/src/offline/idb.ts`
- [x] 3.2 `frontend/src/offline/queryPersist.ts`
- [x] 3.3 `frontend/src/offline/connectivity.ts`
- [x] 3.4 Modo degradado en `SessionProvider.tsx` y `AuthGuard.tsx`
- [x] 3.5 `frontend/src/components/OfflineBanner.tsx`
- [x] 3.6 `pnpm test` en verde

## 4. REFACTOR
- [x] 4.1 `pnpm typecheck`, `pnpm lint` y `pnpm format:check` sin hallazgos
- [x] 4.2 Revisar que no se persiste ningún dato sensible de sesión en IndexedDB

## 5. Cierre
- [x] 5.1 `just ci-ui` y `openspec validate --all --strict` en verde
- [x] 5.2 Actualizar `PLAN.md` §18.4 y `CHANGELOG.md`; archivar el change
