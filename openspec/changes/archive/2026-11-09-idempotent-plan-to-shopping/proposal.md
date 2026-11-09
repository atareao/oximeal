# Proposal

## Why

`POST /api/shopping-list/from-plan` (introducido en #123) es aditivo: fusiona los ingredientes del plan sumando cantidades, así que repetir el volcado duplica las cantidades y editar el plan acumula en vez de reescalar. A diferencia de `from-recipe`, que ya es declarativo, no registra qué entradas del plan se volcaron, por lo que no puede ser idempotente.

## What Changes

- Añadir la tabla `shopping_plan_sources` que registra las entradas de `meal_plan_entries` volcadas a la lista (issue #128).
- Hacer `POST /api/shopping-list/from-plan` **declarativo e idempotente**: añade (unión) las entradas del rango a las fuentes y reconstruye las líneas derivadas del plan reconsolidando la unión.
- Redefinir `POST /api/shopping-list/consolidate` para que **fije** el conjunto de fuentes al rango indicado y reconstruya (conserva su semántica de «reemplazar»).
- Marcar las líneas derivadas del plan con `source_plan_id` no nulo (como ya hace `consolidate`) y **dejar de fusionarlas** con los ítems manuales y las líneas de receta. **BREAKING** visible: un ítem manual «Tomate» y uno del plan «Tomate» conviven como dos líneas, igual que el volcado de receta.
- Retirar el motor aditivo `merge_drafts` y unificar la reconstrucción en un único camino compartido por `from_plan`, `consolidate` y `agent::set_weekly_plan`.
- Actualizar los tests (idempotencia, reescalado, deduplicación de solapes, autorización `write:shopping`).

## Capabilities

### New Capabilities

(Ninguna.)

### Modified Capabilities

- `shopping-list`: el requirement «Volcado aditivo desde el plan» se sustituye por «Volcado declarativo e idempotente desde el plan»; el requirement «Consolidación desde el plan» se matiza para fijar las fuentes al rango.

## Impact

- Backend: nueva migración `backend/migrations/0011_shopping_plan_sources.sql`; `backend/src/repo/shopping_list.rs`; `backend/src/repo/agent.rs`; tests `backend/tests/shopping_list.rs` y `backend/tests/write_transactions.rs`.
- API: mismo contrato HTTP (petición/respuesta); cambia la semántica de `from-plan` (idempotente) y, a nivel de implementación, la de `consolidate`.
- Frontend: sin cambios funcionales (mismo cuerpo de respuesta).
