# Tasks — Verificación de rendimiento y consumo de RAM (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Revisar `justfile` (`build-full`, `ci`, `ci-ui`) y `PLAN.md` §10 (objetivo <20 MB)
- [ ] 1.2 Localizar el smoke E2E con Chrome/Playwright ya usado y el script `frontend/scripts/check-bundle.mjs`
- [ ] 1.3 Fijar y documentar los presupuestos: RSS estable ≤16 MB / pico ≤20 MB, FCP ≤1,5 s, transferencia inicial ≤500 kB, objetivos ≥48px

## 2. RED
- [ ] 2.1 Añadir receta `just` de medición de RSS que falle si los valores superan el presupuesto (aún sin implementación real, en rojo)
- [ ] 2.2 Añadir script de auditoría de rendimiento que falle si FCP/peso incumplen (aún sin medición, en rojo)
- [ ] 2.3 Escribir/documentar la comprobación táctil esperada (objetivos ≥48px; navegación y modo compra con/sin conexión)
- [ ] 2.4 Ejecutar las recetas/scripts y confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Implementar la receta `just` de medición: arranque release con `--features embed-ui`, base temporal, servicio del shell y muestreo de RSS pico/estable
- [ ] 3.2 Implementar el script de auditoría de rendimiento (Playwright/Chrome headless) recogiendo FCP y peso de transferencia inicial en `http://127.0.0.1`
- [ ] 3.3 Implementar la comprobación táctil automatizada o dejar la documentación verificable
- [ ] 3.4 Ejecutar la receta y el script y confirmar que se cumplen todos los presupuestos

## 4. REFACTOR
- [ ] 4.1 `just fmt-check`, `just lint` y `cargo clippy -- -D warnings` en verde
- [ ] 4.2 `pnpm format:check`, `pnpm lint` y `pnpm check-bundle` en verde
- [ ] 4.3 Redactar `docs/performance.md` con metodología, presupuestos, umbrales y resultados

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate perf-device-verification --strict` en verde
- [ ] 5.2 Registrar los resultados reales en `docs/performance.md` y actualizar `PLAN.md` §10 (dejar de marcar la medición como pendiente)
- [ ] 5.3 Anotar la trazabilidad con el issue #47 y archivar el change (`openspec archive`)
