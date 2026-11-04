# Capability: ingredient-consolidation

## ADDED Requirements

### Requirement: Escalado por raciones en el volcado de receta

Al volcar los ingredientes de una receta a la lista, el sistema SHALL escalar
cada cantidad numérica multiplicando por el factor `raciones_solicitadas /
raciones_receta` antes de consolidar, de forma consistente con el escalado
desde el plan. Las raciones de la receta son `recipes.servings` y las
solicitadas el `servings` de la petición.

#### Scenario: Doble de raciones

- GIVEN una receta con `servings = 2` y un ingrediente de 100 g
- WHEN se vuelca con `servings = 4`
- THEN la lista contiene 200 g de ese ingrediente

#### Scenario: Mitad de raciones

- GIVEN una receta con `servings = 4` y un ingrediente de 100 g
- WHEN se vuelca con `servings = 2`
- THEN la lista contiene 50 g de ese ingrediente

#### Scenario: No aditivo al gusto no se escala numéricamente

- GIVEN una receta con "sal" al gusto
- WHEN se vuelca con cualquier `servings`
- THEN la línea de sal se muestra como "al gusto" y sin cantidad

#### Scenario: Familias incompatibles separadas

- GIVEN una receta con "leche" 500 ml y "leche" 1 ud
- WHEN se vuelca
- THEN resultan dos líneas separadas

#### Scenario: Raciones de la receta NULL

- GIVEN una receta con `servings = NULL` y una petición sin `servings`
- WHEN se vuelca
- THEN el factor es 1 y las cantidades no cambian
- AND las raciones de la receta se asumen `2` cuando `servings` es `NULL`
- AND la petición sin `servings` usa las raciones de la receta
