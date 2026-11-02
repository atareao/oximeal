# Capability: ingredient-consolidation

## ADDED Requirements

### Requirement: Escalado por raciones
Al reconstruir la lista de la compra desde el plan, el sistema SHALL escalar las cantidades de cada ingrediente multiplicando por el factor `raciones_planificadas / raciones_receta`, donde las raciones planificadas son `meal_plan_entries.servings`, las de la receta son `recipes.servings` y, si cualquiera es `NULL`, se usa el valor por defecto del esquema (`2`).

#### Scenario: Receta para 2 raciones planificada a 4
- GIVEN una receta con `servings = 2` y un ingrediente de 100 g
- WHEN se planifica una entrada con `servings = 4` para esa receta
- THEN la lista consolidada contiene 200 g de ese ingrediente

#### Scenario: Sin escalado cuando coinciden las raciones
- GIVEN una receta con `servings = 2` y una entrada con `servings = 2`
- WHEN se consolida
- THEN las cantidades no cambian

#### Scenario: Raciones por defecto cuando son NULL
- GIVEN una receta con `servings = NULL` y una entrada con `servings = NULL`
- WHEN se consolida
- THEN el factor es 1 y las cantidades no cambian
