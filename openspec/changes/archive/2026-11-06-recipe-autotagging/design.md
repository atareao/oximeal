# Design — etiquetado automático de recetas

## Context

Ver `proposal.md` — Why. El etiquetado manual ya existe (migración `0007`, `repo::tags::replace_for_recipe`, capacidad `recipe-tags`). Las recetas se crean por tres rutas que convergen en `repo::recipes::create` / `create_or_get_by_source_url`: `POST /api/recipes`, `POST /api/recipes/import` y la ruta de agente (`routes/agent.rs`). El modelo de ingredientes es `IngredientInput { name, amount, unit, category }`; `category` de receta y `diet_type` son campos cerrados existentes. La numeración de migraciones disponible empieza en `0009` (la `0008` está ocupada).

## Goals / Non-Goals

- Goals: inferir un vocabulario cerrado y determinista de etiquetas de dieta, intolerancia y picante; registrar la procedencia de cada asociación; permitir al usuario corregir mediante supresión; reetiquetar lo existente.
- Non-Goals: NLP o IA, etiquetas automáticas de cocina, temporada o coste, búsqueda semántica, y cualquier cambio en el filtro o en el índice FTS.

## Decisions

- **D1 — Vocabulario cerrado embebido.** Las etiquetas automáticas se definen en código (`services::autotag`) con reglas por `slug`; no se configuran por usuario. Alternativa descartada: reglas configurables (complejidad y superficie de error).
- **D2 — Inferencia por ingredientes.** Base: `ingredients[].name`; refuerzo: `name` e `instructions`. Normalización: minúsculas Unicode sin diacríticos, división en tokens no alfanuméricos y coincidencia por token completo (evita que «pan» case dentro de «panqueque»). Las etiquetas de exclusión (`vegetariano`, `vegano`, `sin-gluten`, `sin-lactosa`) se emiten por ausencia de ingredientes vetados; `picante` por presencia. Trade-off: las etiquetas por ausencia se aplican a casi toda receta apta (una ensalada es `sin-gluten`), lo que es correcto pero ruidoso; se mitiga con la supresión por receta.
- **D3 — Procedencia y precedencia.** `recipe_tags.source` con `CHECK (source IN ('manual','auto'))`. La manual gana: si una etiqueta llega en `tags` y además la sugiere la heurística, se guarda una sola fila `manual`. El recálculo solo borra y crea filas `auto`. Alternativa descartada: dos tablas separadas (duplicaría consultas y el filtro). Nota: `PUT /api/recipes/{id}` reemplaza también las supresiones (`suppressed_auto_tags`), por lo que el cliente debe reenviar el estado actual de las supresiones si quiere conservarlas.
- **D4 — Supresión.** Tabla `recipe_tag_suppressions(recipe_id, tag_id)` con clave primaria compuesta y claves foráneas en cascada. El recálculo nunca asigna un `slug` suprimido. Añadir manualmente una etiqueta suprimida borra su supresión.
- **D5 — Punto de integración.** El cálculo se centraliza en `services::autotag::apply(tx, recipe_id, input)` invocado desde `repo::recipes::create` y `update` dentro de la misma transacción `BEGIN IMMEDIATE`, de modo que las tres rutas de alta queden cubiertas. `auto_tags: false` desactiva la parte automática conservando las manuales y las supresiones enviadas.
- **D6 — Backfill.** Tras `0009`, un job idempotente en arranque (`services::autotag::backfill`) recorre las recetas sin filas `auto` y aplica la heurística; un marcador persistente evita repetirlo. Alternativa descartada: backfill en la migración SQL (no puede ejecutar la heurística Rust).
- **D7 — UI.** El formulario muestra las sugerencias como chips retirables; retirar una añade su slug a `suppressed_auto_tags`. Tarjeta y detalle distinguen `auto` (estilo atenuado, «sugerida») de `manual`.

## Risks / Trade-offs

- [Falsos negativos por sinónimos no listados] → listas centralizadas y revisables; ampliarlas junto a los tests.
- [Ruido de etiquetas por ausencia] → supresión por receta y `auto_tags: false`.
- [Backfill costoso en bases grandes] → la base es local y pequeña; el proceso es idempotente y con marcador.
- [Divergencia entre reglas y datos históricos] → `POST /api/recipes/{id}/autotag` permite recalcular.

## Migration Plan

1. Aplicar `0009` (aditiva): columna `source` con `DEFAULT 'manual'`, tabla `recipe_tag_suppressions`, alta idempotente de las cinco etiquetas automáticas del catálogo y el marcador de backfill si aplica.
2. Arranque: ejecutar `backfill` una sola vez.
3. Rollback: DROP de `recipe_tag_suppressions` y de la columna `source` no pierde recetas; las etiquetas manuales permanecen.

## Open Questions

- ¿El vocabulario debe ampliarse en el futuro con `sin-huevo` o `sin-frutos-secos`? No cambia el diseño ni los contratos actuales.
