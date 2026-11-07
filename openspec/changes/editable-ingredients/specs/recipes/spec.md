# Spec Delta

## ADDED Requirements

### Requirement: Desglose de cantidad y unidad al importar
Al importar, SHALL interpretar cada `recipeIngredient` para separar cantidad, unidad y nombre: SHALL reconocer una cantidad inicial (entero, decimal o fracción) y unidades conocidas (p. ej. `g`, `kg`, `ml`, `l`, `ud`, `taza`, `cucharada`, `cucharadita`), guardando `amount` numérico, `unit` normalizada y un `name` sin la cantidad; sin cantidad detectable, SHALL usar `amount = 1`, `unit` "ud" y el nombre completo.

#### Scenario: Cantidad con unidad
- GIVEN un `recipeIngredient` «40 g de queso parmesano»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «queso parmesano», `amount` 40 y `unit` «g»

#### Scenario: Cantidad sin unidad
- GIVEN un `recipeIngredient` «1 calabacín»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «calabacín», `amount` 1 y `unit` «ud»

#### Scenario: Fracción
- GIVEN un `recipeIngredient` «1/2 taza de harina»
- WHEN se importa la receta
- THEN el ingrediente queda con `amount` 0.5, `unit` «taza» y `name` «harina»

#### Scenario: Sin cantidad detectable
- GIVEN un `recipeIngredient` «sal al gusto»
- WHEN se importa la receta
- THEN el ingrediente queda con `amount` 1, `unit` «ud» y `name` «sal al gusto»
