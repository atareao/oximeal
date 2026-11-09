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
El sistema SHALL exponer `PATCH /api/shopping-list/{id}` para alternar `checked`, actualizar `amount` o cambiar `category`. La categoría SHALL validarse contra las categorías canónicas; un valor desconocido SHALL devolver `400`. El cuerpo SHALL contener al menos uno de los tres campos.

#### Scenario: Marcar comprado
- GIVEN un ítem no marcado
- WHEN se hace PATCH con checked=true
- THEN el ítem queda marcado

#### Scenario: Cambiar la categoría
- GIVEN un ítem en la categoría `Otros`
- WHEN se hace PATCH con `category: "Verdulería`"
- THEN el ítem queda en `Verdulería`
- AND `updated_at` se refresca

#### Scenario: Categoría desconocida
- GIVEN un ítem existente
- WHEN se hace PATCH con `category: "Frutas"
- THEN la petición devuelve `400`
- AND el ítem no cambia

#### Scenario: Cuerpo vacío
- GIVEN un ítem existente
- WHEN se hace PATCH sin `checked`, `amount` ni `category`
- THEN la petición devuelve `400`

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

### Requirement: Categorías canónicas
El sistema SHALL usar exactamente estas ocho categorías, en este orden: `Verdulería`, `Pescadería`, `Carnicería`, `Refrigerados`, `Panadería`, `Bebidas`, `Despensa`, `Otros`. La heurística de alta SHALL asignar `Otros` cuando ninguna regla coincida, y el usuario SHALL poder corregir la categoría después.

#### Scenario: Producto sin regla
- GIVEN un nombre que no coincide con ninguna palabra clave
- WHEN se crea el ítem sin categoría explícita
- THEN la categoría es `Otros`

#### Scenario: Corrección manual
- GIVEN un ítem en `Otros`
- WHEN el usuario elige otra categoría en la UI
- THEN la nueva categoría persiste y se muestra agrupada en su pasillo

### Requirement: Volcado aditivo desde el plan

El sistema SHALL exponer `POST /api/shopping-list/from-plan` con permiso `write:shopping` para fusionar los ingredientes del plan de un rango con la lista global. El cuerpo SHALL llevar `start_date` y `end_date` (`YYYY-MM-DD`, `start_date <= end_date`). Las cantidades SHALL escalarse por raciones, consolidarse y fusionarse con los ítems existentes por nombre normalizado y unidad canónica, sumando si la clave coincide. Los ítems existentes SHALL conservarse con sus estados.

#### Scenario: Volcado de un día
- **GIVEN** un plan con entradas para un día y una lista vacía
- **WHEN** se envía `POST /api/shopping-list/from-plan` con `start_date` y `end_date` de ese día
- **THEN** la respuesta `200` contiene la lista agrupada con los ingredientes del día
- **AND** las cantidades se escalan por las raciones de la entrada y de la receta

#### Scenario: Volcado de una semana
- **GIVEN** un plan con entradas de lunes a domingo
- **WHEN** se envía `POST /api/shopping-list/from-plan` con `start_date` lunes y `end_date` domingo
- **THEN** la respuesta contiene los ingredientes consolidados de la semana
- **AND** los ingredientes repetidos se suman

#### Scenario: Fusión con ítems existentes
- **GIVEN** una lista con un ítem manual «Tomate» de 2 ud
- **WHEN** se añade un plan con «Tomate» de 4 ud
- **THEN** la lista conserva el ítem existente con 6 ud
- **AND** no se crea un duplicado

#### Scenario: Conservación de ítems manuales
- **GIVEN** una lista con ítems manuales y volcados de receta
- **WHEN** se añade un plan
- **THEN** los ítems manuales y los volcados se conservan
- **AND** sus estados `checked` y `client_id` no cambian

#### Scenario: Rango inválido
- **GIVEN** un `start_date` posterior a `end_date`
- **WHEN** se envía la petición
- **THEN** se responde `400`
- **AND** la lista no se modifica

#### Scenario: Plan vacío
- **GIVEN** un rango sin entradas en el plan
- **WHEN** se envía la petición
- **THEN** se responde `200` con la lista sin cambios
