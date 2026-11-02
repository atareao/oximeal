# Capability: recipe-search

## ADDED Requirements

### Requirement: Robustez de la búsqueda
El sistema SHALL escapar o citar el término de búsqueda antes de usarlo en `MATCH`, de modo que términos con caracteres especiales de FTS5 (`(`, `"`, `a:b`, operadores colgantes como `AND`/`OR`/`NOT` o prefijos `-`/`*`) NO produzcan error 500; SHALL devolver los resultados coincidentes o una lista vacía.

#### Scenario: Paréntesis de apertura
- GIVEN una petición de listado
- WHEN se consulta `GET /api/recipes?q="("`
- THEN se devuelve 200 con resultados o lista vacía, sin error 500

#### Scenario: Comilla doble
- GIVEN una petición de listado
- WHEN se consulta `GET /api/recipes?q="\""`
- THEN se devuelve 200 con resultados o lista vacía, sin error 500

#### Scenario: Dos puntos y operador colgante
- GIVEN una petición de listado
- WHEN se consulta `GET /api/recipes?q=a:b` o `GET /api/recipes?q=AND`
- THEN se devuelve 200 con resultados o lista vacía, sin error 500

### Requirement: Plegado de diacríticos
El sistema SHALL plegar diacríticos en el índice FTS5 mediante `tokenize='trigram remove_diacritics 1'`, de forma que una búsqueda sin acentos encuentre documentos con acentos.

#### Scenario: Búsqueda sin acento encuentra con acento
- GIVEN una receta cuyo nombre contiene "café"
- WHEN se consulta `GET /api/recipes?q=cafe`
- THEN se devuelve la receta que contiene "café"

### Requirement: Fechas canónicas
El sistema SHALL normalizar toda fecha persistida en `meal_plan_entries.date` a formato canónico `YYYY-MM-DD`, de modo que una fecha no canónica enviada por el cliente se almacene normalizada y sea visible en las consultas de rango.

#### Scenario: Normalización de fecha no canónica
- GIVEN un payload de plan con `date="2026-1-2"`
- WHEN se envía a `POST /api/meal-plan`
- THEN la entrada se persiste con `date="2026-01-02"`

#### Scenario: Fecha normalizada visible en rango
- GIVEN una entrada enviada con `date="2026-1-2"` y ya normalizada
- WHEN se consulta `GET /api/meal-plan?start_date=2026-01-01&end_date=2026-01-31`
- THEN la entrada de "2026-01-02" aparece en los resultados

### Requirement: Validación de ingredientes
El sistema SHALL validar cada ingrediente de `POST /api/recipes`: el nombre SHALL ser no vacío (tras trim) y `amount` SHALL ser mayor o igual que 0; ante un ingrediente inválido SHALL responder 400 sin persistir la receta.

#### Scenario: Ingrediente sin nombre
- GIVEN un payload de receta con un ingrediente de nombre vacío o solo espacios
- WHEN se envía a `POST /api/recipes`
- THEN se devuelve 400 y no se persiste la receta

#### Scenario: Cantidad negativa
- GIVEN un payload de receta con un ingrediente y `amount < 0`
- WHEN se envía a `POST /api/recipes`
- THEN se devuelve 400 y no se persiste la receta

### Requirement: Índices de rendimiento
El sistema SHALL definir los índices `idx_recipes_created_at` sobre `recipes(created_at)`, `idx_shopping_source_plan_id` sobre `shopping_list_items(source_plan_id)` e `idx_shopping_checked` sobre `shopping_list_items(checked)` para acelerar las consultas frecuentes.

#### Scenario: Existencia de índices
- GIVEN una base migrada
- WHEN se consulta `sqlite_master` para el tipo índice
- THEN existen `idx_recipes_created_at`, `idx_shopping_source_plan_id` e `idx_shopping_checked`
