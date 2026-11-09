# Capability: shopping-list

## MODIFIED Requirements

### Requirement: Consolidación desde el plan

El sistema SHALL exponer `POST /api/shopping-list/consolidate` con permiso `write:shopping`, que **fija** el conjunto de entradas del plan consideradas al rango indicado (o la semana ISO actual si no se indica) y **reconstruye** las líneas derivadas del plan reconsolidándolas.

Las líneas derivadas del plan (`source_plan_id` no nulo) SHALL reemplazarse en su totalidad; los ítems manuales y las líneas derivadas de recetas (`source_recipe_id` no nulo) SHALL conservarse sin cambios.

#### Scenario: Consolidación

- GIVEN un plan semanal con ingredientes repetidos
- WHEN se consolida
- THEN la lista resultante contiene cantidades sumadas por ingrediente normalizado

#### Scenario: El volcado de receta sobrevive a la consolidación

- GIVEN líneas derivadas de una receta volcada y líneas derivadas del plan
- WHEN se consolida desde el plan
- THEN las líneas del plan se recalculan por completo
- AND las líneas derivadas de recetas y los ítems manuales permanecen sin cambios

#### Scenario: Fija las fuentes al rango

- GIVEN entradas del plan volcadas previamente desde otro rango
- WHEN se consolida un rango distinto
- THEN las líneas del plan reflejan solo las entradas del rango consolidado

## ADDED Requirements

### Requirement: Volcado declarativo desde el plan

El sistema SHALL exponer `POST /api/shopping-list/from-plan` con permiso `write:shopping`. El cuerpo SHALL llevar `start_date` y `end_date` (`YYYY-MM-DD`, `start_date <= end_date`). La petición SHALL registrar como fuentes las entradas del plan del rango y reconstruir las líneas derivadas del plan (`source_plan_id` no nulo) reconsolidando la unión de entradas registradas.

#### Scenario: Volcado de un día

- GIVEN un plan con entradas para un día y una lista vacía
- WHEN se envía `POST /api/shopping-list/from-plan` con `start_date` y `end_date` de ese día
- THEN la respuesta `200` contiene la lista agrupada con los ingredientes del día marcados con `source_plan_id`
- AND las cantidades se escalan por las raciones de la entrada y de la receta

#### Scenario: Volcado de una semana

- GIVEN un plan con entradas de lunes a domingo
- WHEN se envía `POST /api/shopping-list/from-plan` con `start_date` lunes y `end_date` domingo
- THEN la respuesta contiene los ingredientes consolidados de la semana
- AND los ingredientes repetidos se suman

#### Scenario: Conservación de ítems manuales y de receta

- GIVEN una lista con ítems manuales y volcados de receta
- WHEN se añade un plan
- THEN los ítems manuales y los volcados se conservan
- AND sus estados `checked` y `client_id` no cambian

#### Scenario: Rango inválido

- GIVEN un `start_date` posterior a `end_date`
- WHEN se envía la petición
- THEN se responde `400`
- AND la lista no se modifica

#### Scenario: Plan vacío

- GIVEN un rango sin entradas en el plan
- WHEN se envía la petición
- THEN se responde `200` con la lista sin cambios

#### Scenario: Sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `POST /api/shopping-list/from-plan` sin el permiso `write:shopping`
- THEN se devuelve `401` si no hay sesión o `403` si falta el permiso
- AND no se modifica la lista

### Requirement: Idempotencia y reescalado del volcado del plan

El volcado SHALL ser idempotente: repetir el mismo rango NO SHALL duplicar líneas ni cantidades, y editar el plan y volver a volcar SHALL reescalar sin acumular. Las líneas del plan NO SHALL fusionarse con los ítems manuales ni con las líneas de receta, que SHALL conservarse.

#### Scenario: Repetición idempotente

- GIVEN un plan ya volcado con `from-plan`
- WHEN se repite la misma petición con el mismo rango
- THEN la lista resultante es idéntica
- AND no se duplican líneas ni cantidades

#### Scenario: Añadir otro rango acumula

- GIVEN un plan del que ya se volcó el lunes
- WHEN se vuelca además el martes
- THEN la lista contiene los ingredientes del lunes y del martes
- AND las entradas coincidentes se deduplican por entrada del plan

#### Scenario: Editar el plan reescala sin acumular

- GIVEN un día volcado con una entrada de 2 raciones
- WHEN se cambian las raciones de la entrada y se vuelve a volcar el mismo rango
- THEN las líneas reflejan las nuevas raciones
- AND no permanece sumada la cantidad anterior

#### Scenario: Líneas del plan separadas de las manuales

- GIVEN una lista con un ítem manual «Tomate» de 2 ud
- WHEN se vuelca un plan con «Tomate» de 4 ud
- THEN la lista contiene la línea manual y una línea del plan («Tomate» 4 ud)
- AND no se suman en una sola línea

## REMOVED Requirements

### Requirement: Volcado aditivo desde el plan

**Reason**: sustituido por el volcado declarativo e idempotente; el modelo aditivo duplicaba cantidades al repetir y no reescalaba al editar el plan.

**Migration**: usar `POST /api/shopping-list/from-plan` (mismo contrato de petición y respuesta, nueva semántica idempotente). No hay migración de datos: las líneas del plan se reconstruyen en el primer volcado o consolidación.
