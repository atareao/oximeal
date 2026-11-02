# recipes Specification

## Purpose
Esta capacidad cubre la gestión de recetas: creación, consulta, actualización y borrado (CRUD), así como el listado y la búsqueda de recetas. La búsqueda combina SQLite FTS5 con un fallback por LIKE y permite filtrar por categorías.

## Requirements

### Requirement: Listado y búsqueda de recetas
El sistema SHALL exponer `GET /api/recipes` con filtro de texto opcional `q` (FTS5 trigram) y filtro `category`, con paginación.

#### Scenario: Búsqueda por substring
- GIVEN recetas que contienen "tomate" en nombre o instrucciones
- WHEN se consulta `GET /api/recipes?q=tomate`
- THEN se devuelven las recetas coincidentes con metadatos de paginación

#### Scenario: Query corta
- GIVEN una query de menos de 3 caracteres
- WHEN se consulta el listado
- THEN el sistema usa fallback LIKE en lugar de MATCH

### Requirement: Alta de receta
El sistema SHALL exponer `POST /api/recipes` que valida categoría (main|salad|side|dessert|breakfast) e ingredientes, y persiste la receta.

#### Scenario: Creación válida
- GIVEN un payload de receta válido
- WHEN se envía a `POST /api/recipes`
- THEN se crea la receta con id y se devuelve 201

#### Scenario: Categoría inválida
- GIVEN un payload con categoría no permitida
- WHEN se envía
- THEN se devuelve 400 y no se persiste nada

### Requirement: Detalle de receta
El sistema SHALL exponer `GET /api/recipes/{id}` devolviendo la receta completa con ingredientes.

#### Scenario: No encontrada
- GIVEN un id inexistente
- WHEN se consulta el detalle
- THEN se devuelve 404
