# Capability: shopping-list

## ADDED Requirements

### Requirement: Volcado de una receta a la lista

El sistema SHALL exponer `POST /api/shopping-list/from-recipe` con permiso
`write:shopping` para volcar a la lista global los ingredientes de una receta.
SHALL escalar las cantidades por raciones, consolidarlas con el motor de
`ingredient-consolidation`, fusionarlas en la lista, registrar la receta como
fuente y marcar las líneas resultantes con `source_recipe_id`.

#### Scenario: Volcado feliz

- GIVEN una receta con ingredientes y una lista vacía
- WHEN se envía `POST /api/shopping-list/from-recipe` con `recipe_id` y `servings`
- THEN se crean líneas con cantidades escaladas
- AND la respuesta `200` contiene la lista agrupada resultante (`[ShoppingListGroup]`, mismo cuerpo que `consolidate`) con esas líneas
- AND las líneas quedan marcadas con el `source_recipe_id` de la receta

#### Scenario: Contrato de la petición

- GIVEN una receta existente
- WHEN se envía `POST /api/shopping-list/from-recipe` con `{ "recipe_id": "<uuid>", "servings": <entero >= 1>, opcional }`
- THEN se acepta la petición y se consolida el volcado

#### Scenario: Raciones por defecto

- GIVEN una receta con `servings = 2`
- WHEN se envía la petición sin `servings`
- THEN se usan las raciones de la receta (factor 1)
- AND las cantidades no se alteran

#### Scenario: Raciones inválidas

- GIVEN un `servings` igual a 0, negativo o no entero
- WHEN se envía la petición
- THEN se responde `400`
- AND no se modifica la lista

#### Scenario: Receta inexistente

- GIVEN un `recipe_id` que no existe
- WHEN se envía la petición
- THEN se responde `404`
- AND no se modifica la lista

#### Scenario: Lista global única

- GIVEN una lista con ítems manuales y líneas derivadas del plan
- WHEN se vuelca una receta
- THEN las líneas derivadas del plan y los ítems manuales permanecen
- AND el resultado es una única lista global, no una lista por receta

#### Scenario: Sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `POST /api/shopping-list/from-recipe` sin el permiso `write:shopping`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
- AND no se modifica la lista

### Requirement: Idempotencia y deduplicación del volcado

El sistema SHALL tratar el volcado como una fuente **declarativa**: registrar la
receta y sus raciones en `shopping_recipe_sources` y reconstruir las líneas
derivadas de recetas (`source_recipe_id` no nulo) reconsolidando **todas** las
recetas registradas. Repetir la misma petición SHALL producir la misma lista
sin duplicar líneas. Las líneas derivadas del plan (`source_plan_id`) y los
ítems manuales NO SHALL tocarse.

#### Scenario: Repetición idempotente

- GIVEN una receta ya volcada a la lista
- WHEN se repite la misma petición (`recipe_id` y `servings` idénticos)
- THEN la lista resultante es idéntica
- AND no se duplican líneas

#### Scenario: Cambio de raciones reescala

- GIVEN una receta volcada para 4 raciones
- WHEN se vuelve a volcar la misma receta para 2 raciones
- THEN las líneas derivadas de esa receta reflejan el nuevo factor
- AND no permanece sumada la cantidad anterior

#### Scenario: Fusión entre recetas

- GIVEN dos recetas volcadas que comparten "tomate" en la misma unidad
- WHEN se reconstruyen las líneas derivadas de recetas
- THEN el tomate aparece en una sola línea con las cantidades sumadas
- AND la fusión agrupa por nombre canónico y familia/unidad
- AND se respetan las reglas no aditivas del motor

#### Scenario: No aditivos separados

- GIVEN una receta con "sal" al gusto y otra con "sal" 1 pizca
- WHEN se reconstruyen las líneas derivadas de recetas
- THEN la sal "al gusto" y la sal "1 pizca" permanecen como líneas separadas
- AND no se suman numéricamente entre sí

## MODIFIED Requirements

### Requirement: Consolidación desde el plan

El sistema SHALL exponer `POST /api/shopping-list/consolidate` que reconstruye
las líneas derivadas del plan semanal vigente usando el motor de consolidación,
reemplazando únicamente las líneas con `source_plan_id` no nulo y conservando los
ítems manuales y las líneas derivadas de recetas (`source_recipe_id` no nulo).

#### Scenario: Consolidación

- GIVEN un plan semanal con ingredientes repetidos
- WHEN se consolida
- THEN la lista resultante contiene cantidades sumadas por ingrediente normalizado

#### Scenario: El volcado de receta sobrevive a la consolidación

- GIVEN líneas derivadas de una receta volcada y líneas derivadas del plan
- WHEN se consolida desde el plan
- THEN las líneas del plan se recalculan por completo
- AND las líneas derivadas de recetas y los ítems manuales permanecen sin cambios
