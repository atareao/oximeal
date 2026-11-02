# Tasks — Caché de datos offline y modo degradado (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Revisar `queryKeys` en `frontend/src/api/hooks.ts` y el `QueryClient` de `main.tsx`
- [ ] 1.2 Revisar el manejo de 401 en `client.ts`, `SessionProvider.tsx` y `AuthGuard.tsx`

## 2. RED
- [ ] 2.1 Test: hidratar la caché desde IndexedDB al arrancar
- [ ] 2.2 Test: persistir el resultado de una consulta correcta
- [ ] 2.3 Test: con datos cacheados y sin red, la vista muestra los datos y no redirige al login
- [ ] 2.4 Test: con red y 401, la aplicación redirige al login
- [ ] 2.5 Test: el cierre de sesión limpia la caché de datos de IndexedDB
- [ ] 2.6 Ejecutar `pnpm test`; confirmar el estado RED

## 3. GREEN
- [ ] 3.1 `frontend/src/offline/idb.ts`
- [ ] 3.2 `frontend/src/offline/queryPersist.ts`
- [ ] 3.3 `frontend/src/offline/connectivity.ts`
- [ ] 3.4 Modo degradado en `SessionProvider.tsx` y `AuthGuard.tsx`
- [ ] 3.5 `frontend/src/components/OfflineBanner.tsx`
- [ ] 3.6 `pnpm test` en verde

## 4. REFACTOR
- [ ] 4.1 `pnpm typecheck`, `pnpm lint` y `pnpm format:check` sin hallazgos
- [ ] 4.2 Revisar que no se persiste ningún dato sensible de sesión en IndexedDB

## 5. Cierre
- [ ] 5.1 `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` §18.4 y `CHANGELOG.md`; archivar el change
