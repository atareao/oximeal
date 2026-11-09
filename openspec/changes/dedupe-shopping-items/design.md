# Diseño — reconciliación de líneas duplicadas de la lista de la compra

## Contexto

Tras #129 (`idempotent-plan-to-shopping`) el volcado del plan es declarativo: las líneas derivadas del plan (`source_plan_id` no nulo) se reconstruyen desde `shopping_plan_sources` y las de receta (`source_recipe_id`) desde `shopping_recipe_sources`; ambos orígenes ya deduplican dentro de sí mismos. Lo que queda son las **líneas manuales heredadas** (`source_plan_id IS NULL AND source_recipe_id IS NULL`): el antiguo `from-plan` aditivo las creaba por duplicado al repetir el volcado.

## Objetivos

- Fusionar las líneas manuales con la misma clave, una sola vez, sin tocar los orígenes atribuidos.
- No cambiar el contrato HTTP.
- Garantizar el orden determinista del listado.

## Decisiones

- **D1. Reconciliación de arranque con marcador.** Un servicio `services::shopping_reconcile::reconcile(pool)` se ejecuta en el arranque (como `services::autotag::backfill`), guardado por un marcador persistente en la tabla `shopping_backfills(name PK, applied_at)`. Se descarta hacerlo en una migración SQL porque la clave de fusión requiere canonicalización de unidades (familia + `display_unit`), no expresable en SQL. Se descarta `PRAGMA user_version` para no acoplarse al marcador del autotag.
- **D2. Clave de fusión.** `(nombre normalizado, familia de unidad, unidad de presentación canónica)`, coherente con `domain::consolidate` y con `agent::add_shopping_items`. Solo para líneas manuales.
- **D3. Fila supervivente determinista.** La de `id` lexicográficamente menor; conserva su `name` y su `category`.
- **D4. Cantidades.** Se canonicaliza cada fila (`units::canonicalize`) y se suma en unidades base (redondeo a 2 decimales); el total se expresa en la unidad de la fila supervivente. `ToTaste` resulta en `0.0`; `Unknown` suma si alguna cantidad es distinta de cero y, si no, cuenta ocurrencias (igual que `domain::consolidate`).
- **D5. `checked`.** `1` si alguna de las filas estaba marcada; `0` si no.
- **D6. `client_id`.** El supervivente conserva el suyo; las filas eliminadas no generan conflicto con el índice único parcial (`client_id`) porque se borran.
- **D7. Origen.** Los orígenes distintos (manual / receta / plan) NO se fusionan entre sí.

## Alternativas descartadas

- **Reconciliar en `consolidate`**: cambia la semántica del endpoint («fija fuentes del plan») y exige una acción del usuario; los duplicados persistirían hasta pulsar.
- **Deduplicar en lectura (`GET`)**: una lectura no debe mutar; además `checked` por fila y `PATCH` por `id` quedarían ambiguos.
- **Migración SQL**: no puede agrupar por unidad canónica ni aplicar reglas no aditivas.

## Riesgos y mitigación

- **Dos líneas iguales intencionadas**: se fusionan; el usuario puede volver a separarlas editando (el modelo actual no tiene clave de separación).
- **Cambio de cantidad del supervivente**: se documenta (suma por clave).
- **Marcador**: si la reconciliación falla, no se escribe el marcador y se reintenta en el siguiente arranque; el fallo no impide el arranque.
