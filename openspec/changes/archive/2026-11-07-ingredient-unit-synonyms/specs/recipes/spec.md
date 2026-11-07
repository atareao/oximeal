# Spec Delta

## MODIFIED Requirements

### Requirement: Desglose de cantidad y unidad al importar
Al importar, SHALL interpretar cada `recipeIngredient` para separar cantidad, unidad y nombre: SHALL reconocer una cantidad inicial (entero, decimal o fracción) y una unidad conocida por sus sinónimos normalizados sin conversión de magnitud (masa `g`/`gramos`, `kg`/`kilogramos`/`kilos`; volumen `ml`/`mililitros`, `l`/`litros`; recuento `ud`/`unidad`/`unidades`/`u`; y `taza`, `cucharada`/`cda`, `cucharadita`/`cdta`, `pizca`), guardando `amount` numérico, `unit` normalizada y un `name` sin la cantidad; sin cantidad detectable, SHALL usar `amount = 1`, `unit` "ud" y el nombre completo.

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

#### Scenario: Sinónimo de masa en forma larga
- GIVEN un `recipeIngredient` «150 gramos de pechuga de pollo»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «pechuga de pollo», `amount` 150 y `unit` «g»

#### Scenario: Sinónimo de volumen
- GIVEN un `recipeIngredient` «2 litros de leche»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «leche», `amount` 2 y `unit` «l»

#### Scenario: Recuento en forma larga
- GIVEN un `recipeIngredient` «3 unidades de huevo»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «huevo», `amount` 3 y `unit` «ud»
