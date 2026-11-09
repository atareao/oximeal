# Volcado declarativo e idempotente del plan a la lista

## Qué es
`POST /api/shopping-list/from-plan` vuelca a la lista de la compra los ingredientes del plan de un rango de fechas. Desde el cambio de #128 es **declarativo e idempotente**, igual que `from-recipe`: registra las **entradas del plan** volcadas en la tabla `shopping_plan_sources` y reconstruye las líneas derivadas del plan (`source_plan_id` no nulo) reconsolidando la **unión** de fuentes registradas. Repetir el mismo rango no duplica; añadir otro rango lo une; editar el plan (raciones/receta) reescala sin acumular; los solapes (día dentro de semana) se deduplican por entrada.

Issue: #128 · Change OpenSpec: `idempotent-plan-to-shopping`.

## Ficheros implicados
- Backend
  - `backend/migrations/0011_shopping_plan_sources.sql` — tabla `shopping_plan_sources(entry_id PK → meal_plan_entries(id) ON DELETE CASCADE, created_at)`.
  - `backend/src/repo/shopping_list.rs` — `from_plan`, `consolidate`, `register_plan_sources` (unión), `reset_plan_sources` (fijar), `collect_plan_drafts` (lee la unión de fuentes), `rebuild_plan_lines` y `validate_range`. Retirados `merge_drafts` y `MergeTarget`.
  - `backend/src/repo/agent.rs` — `set_weekly_plan` fija las fuentes al rango cubierto y reconstruye.
  - `backend/src/routes/shopping_list.rs` — handlers `POST /api/shopping-list/from-plan` y `/consolidate` (mismo contrato HTTP).
  - `backend/src/domain/units.rs` — `canonicalize` normaliza la unidad de la familia `Unknown` (`"Cajas"` → `"cajas"`).
  - `backend/tests/shopping_list.rs`, `agent.rs`, `migrations_fts.rs`, `authorization.rs`, `write_transactions.rs`, `shopping_from_recipe.rs` — tests.
- Frontend
  - `frontend/src/api/client.ts` — `addPlanToShoppingList` (sin cambios de contrato).
  - `frontend/src/pages/MenuPage.tsx`, `components/DayCard.tsx`, `components/WeeklyCalendar.tsx`, `components/DailyView.tsx` — botones «Añadir a la lista» (sin cambios funcionales).

## Cómo funciona
- **Fuente = entrada del plan.** La unidad de fuente es `meal_plan_entries.id`, no el rango. La FK `ON DELETE CASCADE` desregistra la fuente si se borra la entrada.
- **`from-plan` = unión.** Hace `INSERT OR IGNORE` de las entradas del rango en `shopping_plan_sources` y reconstruye. Repetir no cambia el conjunto; añadir otro rango lo amplía.
- **`consolidate` = fijar.** Borra todas las fuentes, inserta las del rango y reconstruye (conserva su semántica de «reemplazar»).
- **`agent::set_weekly_plan` = fijar.** Tras reescribir las entradas de la semana, fija las fuentes al rango cubierto y reconstruye (preserva la semántica previa).
- **Reconstrucción.** `rebuild_plan_lines` borra las líneas con `source_plan_id IS NOT NULL` e inserta el resultado de consolidar la unión (`JOIN shopping_plan_sources → meal_plan_entries → recipes`, escalado por raciones) con `source_plan_id = 'consolidated'`.
- **Idempotencia.** Repetir el mismo rango no altera el conjunto de fuentes → misma lista en contenido y cantidades. Las líneas del plan se **recrean con `checked = false`** (no conservan el marcado), igual que en `from-recipe`.
- **Reescalado.** `PATCH /api/meal-plan/{id}` cambia las raciones conservando el id de la entrada; volver a volcar reconstruye con el nuevo factor.
- **Deduplicación de solapes.** Un día y la semana que lo contiene comparten `entry_id`; la fuente es única.
- **Separación de orígenes.** Las líneas del plan **no se fusionan** con los ítems manuales ni con las líneas de receta: conviven (atribución). Un «Tomate» manual de 2 ud y uno del plan de 4 ud son dos líneas.
- **Transacción.** `BEGIN IMMEDIATE` en `from_plan`, `consolidate` y `set_weekly_plan` para evitar `SQLITE_BUSY_SNAPSHOT` en WAL.

## Variables / configuración
Ninguna nueva. El endpoint usa el permiso existente `write:shopping`. La migración `0011_shopping_plan_sources.sql` se aplica automáticamente al arrancar.

## Cómo habilitar / desplegar
- La migración se aplica sola al iniciar el backend (sqlx migrate).
- Sin cambios de contrato HTTP ni de frontend.
- Tras desplegar, el primer `from-plan` o `consolidate` reconstruye las líneas del plan existentes con el nuevo modelo.

## Decisiones
- **D1 — Fuente = entrada del plan.** Deduplica solapes de forma natural y hereda borrado/edición por FK. Alternativa descartada: registrar rangos.
- **D2 — Reconstrucción única.** Un solo camino (`rebuild_plan_lines`) compartido por `from_plan`, `consolidate` y `set_weekly_plan`.
- **D3 — `from-plan` une; `consolidate` fija.** Dos formas de alimentar el mismo conjunto de líneas del plan.
- **D4 — `set_weekly_plan` fija.** Evita acumulación indefinida entre semanas y preserva la semántica previa.
- **D5 — Cambio visible (BREAKING de comportamiento).** Las líneas del plan se separan de los manuales y de las de receta. Aceptado en el diseño (opción A).
- **D6 — Normalización de unidades en el dominio.** `canonicalize` normaliza la unidad `Unknown` para que plan, receta y agente coincidan.

## Follow-ups
- Las líneas del plan se recrean con `checked = false` en cada volcado/consolidación; si se quiere preservar el marcado, habría que conciliar por clave en vez de borrar-y-reinsertar.
- El nombre de presentación del grupo depende del orden de lectura de las entradas (ahora estable por `date, meal_type, role, entry_id`); sinónimos con distinta capitalización podrían mostrar una forma u otra.
- `from-plan` y `consolidate` comparten reconstrucción; el producto podría decidir unificarlos en una sola acción de UI.