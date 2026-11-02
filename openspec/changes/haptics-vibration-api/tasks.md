# Tasks — Haptics como Vibration API (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Confirmar que `useHaptics` usa `navigator.vibrate` y que la spec dice "Web Haptics"
- [ ] 1.2 Revisar los tests actuales de `useHaptics`

## 2. RED
- [ ] 2.1 Añadir test del patrón por defecto (10 ms)
- [ ] 2.2 Añadir test de patrón en array
- [ ] 2.3 Ejecutar `pnpm test` y confirmar el estado de los nuevos tests (RED)

## 3. GREEN
- [ ] 3.1 Actualizar el docstring de `useHaptics.ts` (Vibration API, no-op en iOS Safari)
- [ ] 3.2 Actualizar el delta de `shopping-mode` en el change
- [ ] 3.3 `pnpm test` en verde (GREEN)

## 4. Cierre
- [ ] 4.1 `pnpm typecheck`, `pnpm test`, `pnpm lint` en verde
- [ ] 4.2 `openspec validate haptics-vibration-api --strict` y `openspec validate --all --strict`
- [ ] 4.3 Actualizar `PLAN.md` y `CHANGELOG.md`; archivar el change
