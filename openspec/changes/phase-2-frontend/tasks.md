# Tasks — Fase 2 (TDD: RED → GREEN → REFACTOR)

## 1. Setup
- [ ] 1.1 Scaffold Vite + React 19 + TypeScript en `frontend/`
- [ ] 1.2 Ant Design v6 + `@ant-design/icons` + `ConfigProvider`
- [ ] 1.3 TanStack Query + cliente HTTP base
- [ ] 1.4 ESLint + Prettier + Vitest + Testing Library configurados
- [ ] 1.5 Scripts `dev`, `build`, `test`, `lint`, `typecheck`

## 2. Shell responsive
- [ ] 2.1 `AppShell` con `<Layout.Sider>` en breakpoint desktop
- [ ] 2.2 Bottom Navigation fija (Menú / Compra / Asistente) en móvil
- [ ] 2.3 Rutas SPA y resaltado de pestaña activa
- [ ] 2.4 Tema Antd (tokens) + safe-area insets
- [ ] 2.5 Objetivos táctiles ≥48px en móvil
- [ ] 2.6 Pestaña Asistente abre Valet (`VITE_VALET_URL`)

## 3. Cliente API y hooks
- [ ] 3.1 Tipos TypeScript de recetas, plan y compra
- [ ] 3.2 Cliente `fetch` con manejo de errores y base URL `/api`
- [ ] 3.3 Hooks TanStack Query (meal-plan, shopping-list, mutaciones)
- [ ] 3.4 Tests de hooks con MSW o mock de `fetch`

## 4. Menú semanal
- [ ] 4.1 Cálculo de semana lunes–domingo y navegación entre semanas
- [ ] 4.2 Tarjeta por día con COMIDA y CENA, nombre de receta y marcador de ensalada
- [ ] 4.3 Estados de carga, error (con reintento) y vacío (con acción de añadir)
- [ ] 4.4 Tests de vista semanal

## 5. Modo compra
- [ ] 5.1 Agrupación por pasillo con encabezado y emoji
- [ ] 5.2 Tap para marcar comprado (tachado, opacidad 40%, reordenado al final)
- [ ] 5.3 Vibración con Web Haptics si está disponible
- [ ] 5.4 Alta rápida manual desde input superior
- [ ] 5.5 Acción "Limpiar comprados" (`DELETE /api/shopping-list/completed`)
- [ ] 5.6 Tests de interacción táctil

## 6. Servido embebido
- [ ] 6.1 Feature Cargo `embed-ui` + dependencias `rust-embed`/`mime_guess`
- [ ] 6.2 Handler de assets con MIME correcto y cache para assets con hash
- [ ] 6.3 Fallback SPA a `index.html` para rutas no-`/api`
- [ ] 6.4 Modo sin UI empaquetada: 404 con pista en rutas no-API
- [ ] 6.5 Tests de servido con feature on/off

## 7. Cierre
- [ ] 7.1 `npx tsc --noEmit` y `npm run build` sin errores
- [ ] 7.2 `npx vitest run` en verde
- [ ] 7.3 `eslint` limpio
- [ ] 7.4 `cargo test` en verde con `embed-ui` on y off
- [ ] 7.5 CI actualizado (frontend + backend)
- [ ] 7.6 `openspec validate phase-2-frontend --strict` en verde
