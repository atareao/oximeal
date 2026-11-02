# Change: Fase 1 — Núcleo Backend OxiMeal

## Intent
Construir el núcleo backend self-hosted de OxiMeal: servidor Axum sobre SQLite con recetas, plan semanal y lista de la compra consolidada, incluido el motor determinista de consolidación de ingredientes. Habilita las fases posteriores (UI React, MCP/Chat, PWA offline).

## Scope
- Workspace Cargo con crate `backend` (Axum + Tokio + sqlx/SQLite).
- Migraciones versionadas: `recipes`, `recipes_fts` (FTS5 external-content, trigram), `meal_plan_entries`, `shopping_list_items`.
- Motor de consolidación de ingredientes (normalización de unidades y suma de cantidades).
- API REST de recetas, planificación y lista de la compra.
- Health check + OpenAPI/Swagger.
- Tooling (`justfile`) y cache offline de sqlx.

## Out of scope
- Frontend React (Fase 2), empaquetado rust-embed (Fase 2).
- Servidor MCP y chat SSE (Fase 3).
- PWA / Service Worker / IndexedDB (Fase 4).

## Impact
- Nuevas capacidades: `recipes`, `meal-plan`, `shopping-list`, `ingredient-consolidation`, `server`.
- Introduce persistencia SQLite con WAL y búsqueda FTS5; base para las tools MCP de Fase 3.
