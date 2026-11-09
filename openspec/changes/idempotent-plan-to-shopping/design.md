# Design

## Context

- La lista de la compra es una **única lista semanal global**: no hay una lista por rango ni por plan.
- `from-recipe` ya es declarativo: registra la receta en `shopping_recipe_sources` y reconstruye las líneas con `source_recipe_id`.
- `consolidate` marca las líneas del plan con `source_plan_id = 'consolidated'` y las reemplaza en bloque; `from-plan` (aditivo) las inserta con `source_plan_id` nulo y fusiona por clave.
- `agent::set_weekly_plan` reescribe las entradas de la semana y reconsolida la compra en la misma transacción.
- `foreign_keys=on` (WAL) y `BEGIN IMMEDIATE` para el patrón leer-luego-escribir.

## Goals / Non-Goals

**Goals:**

- Volcado del plan idempotente: repetir el mismo rango no duplica.
- Reescalado: editar el plan y re-volcar recalcula sin acumular.
- Deduplicación de solapes (día ⊂ semana) por entrada del plan.
- Un único mecanismo declarativo de fuentes para el plan.

**Non-Goals:**

- Cifrado en reposo u otras notas del issue #81.
- Cambios de UI (vista mensual, follow-ups de #123/#124).
- Listas múltiples por rango.

## Decisions

- **D1. Fuente = entrada del plan.** Tabla `shopping_plan_sources(entry_id TEXT PRIMARY KEY REFERENCES meal_plan_entries(id) ON DELETE CASCADE, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)`. La unidad de fuente es la entrada (`meal_plan_entries.id`), no el rango: deduplica solapes de forma natural y hereda el borrado/edición por clave foránea. Alternativa descartada: registrar rangos, que exige deduplicar en lectura y gestionar rangos obsoletos.
- **D2. Reconstrucción compartida.** `rebuild_plan_lines(conn)` borra las líneas con `source_plan_id IS NOT NULL` e inserta el resultado de consolidar la unión de fuentes (JOIN `shopping_plan_sources` → `meal_plan_entries` → `recipes`, escalado por raciones, motor de dominio). Sustituye a `replace_consolidated` y `merge_drafts`; se retira `merge_drafts` y `MergeTarget`.
- **D3. `from_plan` = unión.** `from_plan(range)` hace `INSERT OR IGNORE` de las entradas del rango en fuentes y reconstruye. Repetir no cambia el conjunto; añadir otro rango lo amplía; editar el plan reescala al reconstruir.
- **D4. `consolidate` = fijar al rango.** `consolidate(range)` borra todas las fuentes, inserta las del rango y reconstruye. Conserva la semántica actual («reemplazar las líneas del plan por las del rango»).
- **D5. Un solo conjunto de líneas del plan.** Ambas rutas producen `source_plan_id = 'consolidated'`; no hay dos conjuntos compitiendo.
- **D6. `agent::set_weekly_plan` = unión.** Tras reescribir las entradas (borra e inserta), registra las del rango con `INSERT OR IGNORE` y reconstruye; las fuentes de otras fechas se conservan. Sustituye su uso directo de `collect_drafts` + `replace_consolidated`.
- **D7. Las líneas del plan ya no se fusionan con las manuales.** Inherente al modelo declarativo (atribución), igual que `from-recipe`. Un ítem manual «Tomate» y uno del plan «Tomate» quedan como dos líneas.

## Risks / Trade-offs

- **Cambio de comportamiento visible (BREAKING)**: `from-plan` deja de sumar sobre ítems manuales/receta; las líneas del plan quedan atribuidas y separadas. Documentado en la spec y en `docs/`.
- **Datos heredados**: las líneas antiguas del plan (`source_plan_id='consolidated'`) se regeneran en el primer `from-plan`/`consolidate`; solo hay migración de esquema, no de datos.
- **Dependencia de FK**: `shopping_plan_sources` requiere `foreign_keys=on` (ya activo).
