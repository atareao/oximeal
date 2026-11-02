# shopping-list Specification

## Purpose
Esta capacidad gestiona la lista de la compra derivada del plan semanal. Agrupa los ítems por pasillo, permite marcar y desmarcar su estado de compra, consolida los ingredientes desde el plan y purga los ítems ya comprados.

## Requirements

### Requirement: Lista agrupada
El sistema SHALL exponer `GET /api/shopping-list` devolviendo los ítems agrupados por categoría de pasillo y con su estado `checked`.

#### Scenario: Agrupación
- GIVEN ítems en varias categorías
- WHEN se consulta la lista
- THEN la respuesta se agrupa por categoría con los ítems dentro

### Requirement: Toggle y edición
El sistema SHALL exponer `PATCH /api/shopping-list/{id}` para alternar `checked` o actualizar cantidad.

#### Scenario: Marcar comprado
- GIVEN un ítem no marcado
- WHEN se hace PATCH con checked=true
- THEN el ítem queda marcado

### Requirement: Consolidación desde el plan
El sistema SHALL exponer `POST /api/shopping-list/consolidate` que reconstruye la lista a partir del plan semanal vigente usando el motor de consolidación.

#### Scenario: Consolidación
- GIVEN un plan semanal con ingredientes repetidos
- WHEN se consolida
- THEN la lista resultante contiene cantidades sumadas por ingrediente normalizado

### Requirement: Purga de comprados
El sistema SHALL exponer `DELETE /api/shopping-list/completed` que elimina los ítems marcados.

#### Scenario: Purga
- GIVEN ítems marcados y no marcados
- WHEN se purga
- THEN solo permanecen los no marcados

### Requirement: Alta idempotente por identificador de cliente
`POST /api/shopping-list` SHALL aceptar un campo opcional `client_id`; si ya existe un ítem con ese `client_id`, la petición SHALL devolver el ítem existente sin crear uno nuevo.

#### Scenario: Alta normal sin client_id
- GIVEN una petición de alta sin `client_id`
- WHEN se procesa
- THEN se crea un ítem nuevo

#### Scenario: Alta repetida con el mismo client_id
- GIVEN un ítem creado con `client_id` `"abc"`
- WHEN se repite el alta con el mismo `client_id`
- THEN no se crea un duplicado
- AND se devuelve el ítem existente
