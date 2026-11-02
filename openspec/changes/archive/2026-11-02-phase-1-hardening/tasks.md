# Tasks — Fase 1 Hardening (TDD: RED → GREEN → REFACTOR)

## 1. Búsqueda robusta (FTS-escape)
- [x] 1.1 RED: test de integración `GET /api/recipes?q=(` → 200 (sin 500)
- [x] 1.2 RED: tests con `"`, `a:b`, `AND` y operadores colgantes → 200
- [x] 1.3 GREEN: escapar/citar el término antes de `MATCH`
- [x] 1.4 REFACTOR: helper único de escapado reutilizado por búsqueda y Agent API

## 2. Plegado de diacríticos
- [x] 2.1 RED: test `q=cafe` encuentra receta con "café"
- [x] 2.2 GREEN: migración con `tokenize='trigram remove_diacritics 1'` + reindexado
- [x] 2.3 REFACTOR: verificar índice y reconstrucción en migración

## 3. Fechas canónicas
- [x] 3.1 RED: test POST con `2026-1-2` se normaliza a `2026-01-02`
- [x] 3.2 RED: test de que la entrada normalizada aparece en la consulta de rango
- [x] 3.3 GREEN: normalizar `YYYY-MM-DD` antes de persistir en `meal_plan_entries.date`
- [x] 3.4 REFACTOR: centralizar el parseo/normalización de fechas

## 4. Validación de ingredientes
- [x] 4.1 RED: test `POST /api/recipes` con ingrediente sin nombre → 400
- [x] 4.2 RED: test con `amount < 0` → 400 y sin persistir
- [x] 4.3 GREEN: validar nombre no vacío y `amount >= 0` por ingrediente
- [x] 4.4 REFACTOR: extraer validador de ingredientes reutilizable

## 5. Índices de rendimiento
- [x] 5.1 RED: test que verifica la existencia de `idx_recipes_created_at`, `idx_shopping_source_plan_id`, `idx_shopping_checked`
- [x] 5.2 GREEN: migración que crea los índices faltantes
- [x] 5.3 REFACTOR: confirmar que los planes de consulta usan los índices

## 6. Cierre
- [x] 6.1 cargo test / clippy -D warnings / fmt --check en verde
- [x] 6.2 openspec validate phase-1-hardening --strict en verde
