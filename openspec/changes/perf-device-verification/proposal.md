# Change: verificación de rendimiento y consumo de RAM en dispositivo real

## Why

La Fase 5 cerró con la verificación funcional y el smoke E2E, pero dejó
explícitamente **pendiente "pruebas táctiles y de rendimiento"**. En concreto,
`PLAN.md` §10 registra el objetivo global **consumo de RAM <20 MB**, marcado
como **pendiente de medir sobre el binario monolítico** (`cargo build --release
-p oximeal-backend --features embed-ui`): hoy no existe ninguna medición ni
receta reproducible que demuestre que el servidor con la UI embebida se mantiene
por debajo de ese techo. Tampoco hay presupuestos web medidos (primer contenido
útil, peso de la transferencia inicial) ni una comprobación sistemática de los
objetivos táctiles ≥48px y del comportamiento de las interacciones clave con y
sin conexión. Issue de trazabilidad: **#47**.

## What Changes

- Nueva receta `just` de **medición de RSS** del binario release con la UI
  embebida: arranque contra una base temporal, servicio del shell y muestreo de
  RSS **pico** y **estable** (p. ej. leyendo `/proc/<pid>/status`).
- Script de **auditoría de rendimiento** reutilizando el navegador headless
  (Playwright/Chrome) ya usado para el smoke E2E, para medir los presupuestos
  web (primer contenido útil y peso de transferencia inicial) sobre
  `http://127.0.0.1`.
- Verificación **táctil**: comprobación automatizada o documentada de objetivos
  ≥48px en los controles principales y de la respuesta de las interacciones
  clave (navegación, modo compra) con y sin conexión.
- `docs/performance.md` con la metodología, los presupuestos fijados y los
  resultados obtenidos.

## Impact

- Capabilities: nueva `performance`.
- Tooling: `justfile` (nuevas recetas), `docs/performance.md`, posible script
  en `scripts/` (auditoría de rendimiento) y/o `frontend/scripts/`.
- Sin cambios de contrato con el backend ni dependencias de runtime nuevas.
