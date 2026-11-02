# Change: Fase 2 — Frontend React 19 + Antd v6 (PWA-first)

## Intent
Dotar a OxiMeal de su interfaz web mobile-first (React 19 + Ant Design v6, Vite + TypeScript) que consume la API REST y la Agent API existentes, y servirla como binario único mediante `rust-embed`.

## Scope
- App Vite + React 19 + TypeScript + Antd v6 en `frontend/`.
- Shell responsive: `<Layout.Sider>` con panel lateral en desktop; Bottom Navigation (Menú / Compra / Asistente) en móvil.
- Vista de menú semanal (calendario lunes–domingo con comida y cena).
- Vista de lista de la compra en modo supermercado (agrupada por pasillo, objetivos ≥48px, feedback táctil).
- Cliente API tipado + hooks de datos (TanStack Query).
- Servido estático con `rust-embed` (feature `embed-ui`) y fallback SPA para rutas no-`/api`.
- La pestaña Asistente enlaza/embebe Valet (no hay chat nativo).

## Out of scope
- Service Worker / IndexedDB / offline (Fase 4).
- Chat, orquestador LLM y MCP (decisión: el asistente es Valet).
- Autenticación.

## Impact
Nuevas capacidades `ui-shell`, `weekly-menu`, `shopping-mode` y `frontend-serving`. Nuevos assets en `frontend/` y un módulo de servido estático en el backend.
