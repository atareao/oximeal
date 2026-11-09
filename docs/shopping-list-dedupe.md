# Reconciliación de duplicados de la lista de la compra y orden determinista

## Qué es
Al arrancar, el backend ejecuta una **reconciliación de un solo uso** que fusiona las **líneas manuales duplicadas** de la lista de la compra: filas con la misma clave (nombre normalizado + familia y unidad canónica) que quedaron duplicadas por el antiguo `from-plan` aditivo, que insertaba las líneas del plan como si fueran ítems manuales. La reconciliación suma las cantidades, conserva una fila determinista y elimina el resto. Además, el listado tiene un **orden determinista** garantizado por test.

Issue: #131 · Change OpenSpec: `dedupe-shopping-items`.

## Ficheros implicados
- `backend/migrations/0012_shopping_backfills.sql` — tabla `shopping_backfills(name PK, applied_at)` con las reconciliaciones ya aplicadas.
- `backend/src/services/shopping_reconcile.rs` — `reconcile(pool)` y la constante `MARK_MERGE_DUPLICATE_LINES`.
- `backend/src/services/mod.rs` — registro del módulo.
- `backend/src/main.rs` — llamada al servicio tras el backfill de autotag, tolerante a fallos.
- `backend/src/repo/shopping_list.rs` — `list_grouped` ya ordena por `name COLLATE NOCASE ASC, id ASC`.
- `backend/tests/shopping_reconcile.rs` — 15 tests del servicio.
- `backend/tests/shopping_list.rs` — test de orden determinista del listado.
- `backend/tests/migrations_fts.rs` — incluye `shopping_backfills` entre las tablas esperadas.

## Cómo funciona
- **Solo líneas manuales.** Se leen las filas con `source_plan_id IS NULL AND source_recipe_id IS NULL`; las líneas atribuidas (receta/plan) no se tocan porque ya se reconstruyen deduplicadas desde sus fuentes.
- **Clave de fusión:** `(normalize_name(name), familia de unidad, display_unit)` vía `domain::units::canonicalize`, coherente con `domain::consolidate` y `agent::add_shopping_items`.
- **Supervivente determinista:** la de `id` lexicográficamente menor; conserva su nombre y categoría.
- **Cantidades:** se suman en unidades base (redondeo a 2 decimales) y se reexpresan en la unidad del supervivente. No aditivos (`ToTaste`) → `0.0`; `Unknown` → suma si alguna cantidad es distinta de cero y, si no, cuenta ocurrencias (igual que `domain::consolidate`).
- **`checked`:** `1` si alguna fila estaba marcada.
- **`client_id`:** el supervivente conserva el suyo; las filas eliminadas no crean conflicto con el índice único parcial porque se borran.
- **Marcador e idempotencia:** todo ocurre en una transacción `BEGIN IMMEDIATE`; al final se inserta la marca. Si la marca ya existe, no se hace nada. Si falla, no se escribe la marca y se reintenta en el siguiente arranque; el fallo no impide el arranque.
- **Orden del listado:** `GET /api/shopping-list` devuelve los ítems ordenados por `name COLLATE NOCASE ASC, id ASC` (el frontend los reordena además por pendientes/`updated_at`).

## Variables / configuración
Ninguna nueva. No cambia el contrato HTTP ni el frontend.

## Cómo habilitar / desplegar
- La migración `0012` se aplica sola al iniciar el backend (sqlx migrate).
- La reconciliación se ejecuta automáticamente en el primer arranque tras desplegar y no vuelve a repetirse.
- Para forzar una nueva ejecución (p. ej. tras restaurar datos duplicados a mano) basta con borrar la fila del marcador: `DELETE FROM shopping_backfills WHERE name = 'merge-duplicate-lines';` y reiniciar.

## Decisiones
- **Reconciliación de arranque con marcador** en vez de migración SQL: la clave requiere canonicalizar unidades (familia + presentación), no expresable en SQL.
- **Tabla `shopping_backfills`** en vez de `PRAGMA user_version`: no se acopla al backfill de autotag y permite acumular reconciliaciones independientes.
- **Solo manuales:** los orígenes atribuidos ya deduplican por fuente; fusionarlos entre sí rompería el modelo declarativo de #129.
- **Deduplicar al leer, descartado:** una lectura no debe mutar y dejaría ambiguos `checked` y `PATCH` por `id`.
- **`consolidate`, descartado:** cambia su semántica de «fijar fuentes del plan» y exigiría una acción del usuario.

## Follow-ups
- Las líneas iguales **intencionadas** se fusionan; el modelo actual no tiene clave de separación.
- La fusión pierde los `client_id` de las filas eliminadas (solo relevante para reintentos en vuelo).
- Sigue pendiente la acción «Añadir a la lista» en la **vista mensual** (follow-up de #123).