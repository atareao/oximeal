# Reconciliar líneas duplicadas heredadas de la lista de la compra

## Por qué

La feature #123 (volcado del plan a la lista) tenía un `from-plan` **aditivo** que insertaba los ingredientes como **ítems manuales** (`source_plan_id = NULL`). Repetir el volcado no era idempotente y dejaba **filas duplicadas con la misma clave** (nombre normalizado + unidad canónica), ya indistinguibles de los ítems manuales. El change #128/#129 (`idempotent-plan-to-shopping`) corrigió el comportamiento hacia adelante, pero **no limpia los duplicados ya existentes** en la base de datos.

La auditoría de #124 también señaló que la fusión era **no determinista** (`SELECT_ITEM` sin `ORDER BY` en `merge_drafts`); ese motor se retiró en #129, y ahora hay que **garantizar y testear** que el listado tiene un orden determinista.

Issue: #131

## Qué cambia

- **Reconciliación única al arrancar** (`services::shopping_reconcile`) que fusiona las líneas **manuales** (`source_plan_id IS NULL AND source_recipe_id IS NULL`) que comparten la misma clave de fusión. Marcador persistente en una tabla nueva `shopping_backfills` para ejecutarse una sola vez.
- **Orden determinista** del listado (`name COLLATE NOCASE ASC, id ASC`) explicitado en la spec y cubierto por un test.
- Migración `0012_shopping_backfills.sql` con la tabla del marcador.

## Impacto

- **Backend**: nueva migración, servicio de reconciliación, llamada en el arranque y tests.
- **Frontend**: sin cambios.
- **Contrato HTTP**: sin cambios.
- **Datos**: en el primer arranque tras desplegar, las líneas manuales duplicadas se fusionan (suma de cantidades). Idempotente y con marcador: no se repite.
