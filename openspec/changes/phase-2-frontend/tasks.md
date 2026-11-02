# Tasks — Fase 2 (TDD: RED → GREEN → REFACTOR)

## 1. Setup
- [x] 1.1 Scaffold Vite + React 19 + TypeScript en `frontend/`
- [x] 1.2 Ant Design v6 + `ConfigProvider` (sin `@ant-design/icons`: dependencia eliminada por no usarse)
- [x] 1.3 TanStack Query + cliente HTTP base
- [x] 1.4 ESLint + Vitest + Testing Library configurados
- [ ] 1.4b Prettier — APLAZADO: no se añadió (deuda diferida; sin `eslint-config-prettier`)
- [x] 1.5 Scripts `dev`, `build`, `preview`, `test`, `lint`, `typecheck`

## 2. Shell responsive
- [x] 2.1 `AppShell` con `<Layout.Sider>` en breakpoint desktop
- [x] 2.2 Bottom Navigation fija (Menú / Compra / Asistente) en móvil
- [x] 2.3 Rutas SPA y resaltado de pestaña activa
- [x] 2.4 Tema Antd (tokens) + safe-area insets
- [x] 2.5 Objetivos táctiles ≥48px en móvil
- [x] 2.6 Pestaña Asistente abre Valet (`VITE_VALET_URL` vía `src/lib/config.ts`)

## 3. Cliente API y hooks
- [x] 3.1 Tipos TypeScript de recetas, plan y compra
- [x] 3.2 Cliente `fetch` con manejo de errores y base URL `/api`
- [x] 3.3 Hooks TanStack Query (meal-plan, shopping-list, mutaciones)
- [x] 3.4 Tests de hooks/componentes (sin MSW: se mockea `src/api/client` con `vi.mock` y se usan hooks reales con `QueryClientProvider`)

## 4. Menú semanal
- [x] 4.1 Cálculo de semana lunes–domingo y navegación entre semanas
- [x] 4.2 Tarjeta por día con COMIDA y CENA, nombre de receta y marcador de ensalada
- [x] 4.3 Estados de carga, error (con reintento) y vacío (con acción "Añadir comidas con Valet" si `VALET_URL` está configurada)
- [x] 4.4 Tests de vista semanal

## 5. Modo compra
- [x] 5.1 Agrupación por pasillo con encabezado y emoji
- [x] 5.2 Tap para marcar comprado (tachado, opacidad 40%, reordenado al final)
- [x] 5.3 Vibración con la Vibration API (`navigator.vibrate`) en `src/hooks/useHaptics.ts` (NO se usó el paquete `web-haptics`)
- [x] 5.4 Alta rápida manual desde input superior
- [x] 5.5 Acción "Limpiar comprados" (`DELETE /api/shopping-list/completed`)
- [x] 5.6 Tests de interacción táctil

## 6. Servido embebido
- [x] 6.1 Feature Cargo `embed-ui` + dependencias `rust-embed`/`mime_guess`
- [x] 6.2 Handler de assets con MIME correcto y cache inmutable para assets con hash
- [x] 6.3 Fallback SPA a `index.html` para rutas no-`/api`
- [x] 6.4 Modo sin UI empaquetada: 404 con pista en rutas no-API
- [x] 6.5 Tests de servido con feature on/off

## 7. Cierre
- [x] 7.1 `pnpm typecheck` y `pnpm build` sin errores (se usó **pnpm**, no `npx`/`npm`)
- [x] 7.2 `pnpm test` en verde (73 tests)
- [x] 7.3 `eslint` limpio (0 problemas)
- [x] 7.4 `cargo test` en verde con `embed-ui` on y off (on: 63 unit + 67 integración; off: 48 unit + 53 integración)
- [x] 7.5 CI local vía `just` actualizado (frontend + backend): `just ci` incluye `test-embed-ui` y `just ci-ui` cubre el frontend. **Sin workflow de GitHub Actions** (pendiente).
- [x] 7.6 `openspec validate phase-2-frontend --strict` en verde

## Notas de implementación

### Bundle (build de producción verificado)
- Entry `index-*.js`: ~8.4 kB (`8421` bytes).
- Vendor `antd-*.js`: 683 kB (`683349` bytes); ~213 kB gzip (medido con `gzip -c | wc -c` → `218103` bytes). Comparte chunk React/Query separados.
- **Code-splitting por rutas activo**: `MenuPage-*.js` (~4.8 kB) y `ShoppingPage-*.js` (~6.1 kB) se emiten como chunks independientes junto a sus CSS.

### Deuda diferida (registrada como issues, no bloqueante)
- ETag / `304 Not Modified` para assets estáticos.
- `Content-Type` con `charset=utf-8`.
- Cabeceras HTTP: `Accept-Ranges` / `Range`, `X-Content-Type-Options: nosniff`, CSP.
- Plugin `eslint-plugin-jsx-a11y` (accesibilidad lint del JSX).
- Prettier (ver 1.4b).
- Workflow de GitHub Actions para CI remota (hoy CI es local vía `just`).
