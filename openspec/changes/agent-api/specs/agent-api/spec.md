# Capability: agent-api

## ADDED Requirements

### Requirement: Recetas recientes
El sistema SHALL exponer `GET /api/agent/recent-recipes?days=` devolviendo las recetas planificadas en los últimos N días junto con su última fecha de planificación, ordenadas de más reciente a más antigua.

#### Scenario: Hay datos
- GIVEN recetas planificadas en los últimos N días
- WHEN se consulta `GET /api/agent/recent-recipes?days=N`
- THEN se devuelven las recetas con su última fecha de planificación

#### Scenario: Sin datos
- GIVEN un plan vacío en los últimos N días
- WHEN se consulta `GET /api/agent/recent-recipes?days=N`
- THEN se devuelve una lista vacía sin error

### Requirement: Búsqueda para el agente
El sistema SHALL exponer `GET /api/agent/search-recipes?query=&category=` reutilizando la búsqueda FTS5 con fallback LIKE para queries cortas y SHALL limitar el número de resultados devueltos.

#### Scenario: Coincidencia
- GIVEN recetas que contienen el término buscado
- WHEN se consulta `GET /api/agent/search-recipes?query=tomate`
- THEN se devuelven las coincidencias hasta el límite configurado

#### Scenario: Query corta
- GIVEN una query de menos de 3 caracteres
- WHEN se consulta la búsqueda
- THEN el sistema usa el fallback LIKE en lugar de MATCH

### Requirement: Fijar plan semanal
El sistema SHALL exponer `POST /api/agent/weekly-plan` que valida TODAS las entradas antes de escribir, persiste de forma transaccional, reemplaza las entradas de cada fecha cubierta y reconstruye la lista de la compra conservando los ítems manuales.

#### Scenario: Todas las entradas válidas
- GIVEN un conjunto de entradas con `recipe_id` existentes y fechas válidas
- WHEN se envía a `POST /api/agent/weekly-plan`
- THEN las entradas de las fechas cubiertas quedan reemplazadas y la compra se reconstruye conservando los ítems manuales

#### Scenario: Entrada inválida
- GIVEN una entrada con un `recipe_id` que no existe
- WHEN se envía a `POST /api/agent/weekly-plan`
- THEN se devuelve 400 sin persistir ninguna entrada

### Requirement: Añadir a la compra
El sistema SHALL exponer `POST /api/agent/shopping-items` que suma la cantidad a un ítem existente con la misma clave normalizada y la misma unidad canónica, o inserta uno nuevo con categoría heurística.

#### Scenario: Suma a ítem existente
- GIVEN un ítem existente con la misma clave normalizada y unidad canónica
- WHEN se envía `POST /api/agent/shopping-items` con una cantidad adicional
- THEN la cantidad del ítem existente se incrementa

#### Scenario: Inserción de ítem nuevo
- GIVEN que no existe un ítem con esa clave normalizada
- WHEN se envía `POST /api/agent/shopping-items`
- THEN se inserta un nuevo ítem con categoría heurística

### Requirement: Purgar comprados
El sistema SHALL exponer `DELETE /api/agent/shopping/completed` que elimina los ítems marcados como comprados y devuelve el número de ítems eliminados.

#### Scenario: Purga
- GIVEN ítems marcados y no marcados
- WHEN se llama a `DELETE /api/agent/shopping/completed`
- THEN solo permanecen los no marcados y se devuelve el número eliminado
