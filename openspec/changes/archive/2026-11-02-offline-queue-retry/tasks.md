# Tasks — Reintento del drenado de la cola (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Leer `frontend/src/offline/MutationQueueProvider.tsx`, `syncShoppingQueue.ts` y `mutationQueue.ts` (`DrainResult`)
- [x] 1.2 Reproducir el fallo: con `createItem` que falla por red y luego funciona, comprobar que la cola no se reintenta sola

## 2. RED
- [x] 2.1 Test: `drainScheduler` reprograma el drenado con espera creciente tras `stoppedBy: 'network'` (temporizadores falsos)
- [x] 2.2 Test: el intervalo está acotado por el máximo
- [x] 2.3 Test: con la cola vacía no se programa reintento y el backoff se reinicia
- [x] 2.4 Test: tras progresar (`processed > 0` y pendientes) se reintenta de inmediato
- [x] 2.5 Test: un 401 no provoca reintento en bucle
- [x] 2.6 Test: `dispose()` cancela el temporizador pendiente
- [x] 2.7 Test de integración: el provider reintenta y termina drenando la cola cuando el fallo de red deja de producirse
- [x] 2.8 Ejecutar `pnpm test` y confirmar el estado RED

## 3. GREEN
- [x] 3.1 `frontend/src/offline/drainScheduler.ts`
- [x] 3.2 `MutationQueueProvider` usa el planificador y reintenta en `visibilitychange`/`focus`
- [x] 3.3 `pnpm typecheck && pnpm test` en verde

## 4. REFACTOR
- [x] 4.1 `pnpm format:check`, `pnpm lint` y `pnpm check-bundle` (presupuesto 500 kB)

## 5. Cierre
- [x] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [x] 5.2 Re-verificar con el smoke E2E en navegador real que la cola se drena sola al recuperar la conexión (37/37)
- [x] 5.3 Actualizar `PLAN.md` §18.8 y `CHANGELOG.md`; archivar el change
