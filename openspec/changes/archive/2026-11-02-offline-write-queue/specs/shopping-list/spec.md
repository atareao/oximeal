# Capability: shopping-list

## ADDED Requirements

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
