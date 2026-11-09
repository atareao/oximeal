# Spec Delta

## ADDED Requirements

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