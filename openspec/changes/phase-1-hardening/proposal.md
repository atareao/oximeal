# Change: Fase 1 — Hardening del núcleo backend

## Intent
Endurecer el núcleo backend de Fase 1 ante los defectos detectados en auditoría (rust-reviewer + sqlite-expert): búsqueda FTS5 no robusta ante caracteres especiales, fechas no canónicas en el plan, diacríticos no plegados y validación de ingredientes incompleta. El objetivo es que la API devuelva respuestas estables (sin 500) y datos normalizados, y que las consultas frecuentes estén indexadas.

## Scope
- Escapado/citado del término de búsqueda antes de `MATCH` (caracteres especiales de FTS5: `(`, `"`, `a:b`, operadores colgantes).
- Plegado de diacríticos en el índice FTS5 (`tokenize='trigram remove_diacritics 1'`).
- Normalización de fechas a `YYYY-MM-DD` antes de persistir en `meal_plan_entries.date`.
- Validación de ingredientes en `POST /api/recipes` (nombre no vacío, `amount >= 0`).
- Índices de rendimiento: `idx_recipes_created_at`, `idx_shopping_source_plan_id`, `idx_shopping_checked`.

## Out of scope
- Reescritura de la superficie de la API o de los contratos de recetas/plan/compra (se mantienen los de `phase-1-core-backend`).
- Cambios en la Agent API (`agent-api`).
- Frontend React (Fase 2), MCP/chat (Fase 3), PWA/Offline (Fase 4).

## Impact
- Endurece las capacidades `recipes` y `meal-plan`; añade verificación de índices sobre el esquema existente.
- Sin cambios de contrato HTTP; sí en robustez, normalización de datos y planes de consulta.
- Requiere nueva migración para la contribución de diacríticos del tokenizer y los índices.
