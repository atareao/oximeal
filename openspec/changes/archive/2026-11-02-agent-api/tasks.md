# Tasks — Agent API (TDD: RED → GREEN → REFACTOR)

## 1. Contrato
- [x] 1.1 DTOs serde + `ToSchema` de la Agent API (`recent-recipes`, `search-recipes`, `weekly-plan`, `shopping-items`, purga)
- [x] 1.2 Registrar las rutas `/api/agent` en el router factoría y en OpenAPI

## 2. Endpoints de lectura
- [x] 2.1 `GET /api/agent/recent-recipes?days=` (recetas planificadas + última fecha)
- [x] 2.2 `GET /api/agent/search-recipes?query=&category=` (FTS5/LIKE + límite)

## 3. Endpoints de escritura
- [x] 3.1 `POST /api/agent/weekly-plan` (validación total, transacción, reemplazo por fecha, reconstrucción de compra)
- [x] 3.2 `POST /api/agent/shopping-items` (suma por clave normalizada + unidad canónica o inserción con categoría heurística)
- [x] 3.3 `DELETE /api/agent/shopping/completed` (purga + nº eliminado)

## 4. Tests de integración (RED → GREEN)
- [x] 4.1 recent-recipes con datos y sin datos
- [x] 4.2 search-recipes match y query corta (LIKE)
- [x] 4.3 weekly-plan válido y recipe_id inexistente (400 sin persistir)
- [x] 4.4 shopping-items suma e inserción
- [x] 4.5 shopping/completed purga y recuento

## 5. Cierre
- [x] 5.1 Documentar los 5 endpoints en OpenAPI/Swagger
- [x] 5.2 cargo test / clippy -D warnings / fmt --check en verde
- [x] 5.3 openspec validate agent-api --strict en verde
