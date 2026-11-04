# ingredient-consolidation Specification

## Purpose
Esta capacidad implementa el motor determinista de consolidación de ingredientes. Canonicaliza y normaliza unidades, suma cantidades compatibles y categoriza los ingredientes para producir listas de compra reproducibles.

## Requirements

### Requirement: Canonicalización de unidades
El sistema SHALL convertir cantidades a unidades canónicas por familia física (masa→gramos, volumen→mililitros) y SHALL distinguir familias no aditivas (pizca, al gusto, unidades).

#### Scenario: Conversión de masa
- GIVEN un ingrediente con cantidad 1.5 y unidad "kg"
- WHEN se canonicaliza
- THEN el resultado es 1500 gramos de la familia Masa

#### Scenario: Unidad desconocida
- GIVEN un ingrediente con unidad "puñado"
- WHEN se canonicaliza
- THEN se clasifica como familia Count/Pinch sin conversión numérica

### Requirement: Normalización de nombres
El sistema SHALL normalizar nombres (minúsculas, sin diacríticos, sin artículos, singular básico) para agrupar ingredientes equivalentes.

#### Scenario: Acentos y artículo
- GIVEN los nombres "Cebollas" y "la cebolla"
- WHEN se normalizan
- THEN ambos producen la misma clave normalizada

### Requirement: Suma de cantidades
El sistema SHALL sumar cantidades de ingredientes con la misma clave normalizada y la misma familia de unidad, y SHALL mantener líneas separadas si las familias difieren o son no aditivas.

#### Scenario: Suma dentro de la misma familia
- GIVEN "tomate" 2 unidades y "tomates" 3 unidades
- WHEN se consolida
- THEN resulta una línea con 5 unidades de tomate

#### Scenario: Familias incompatibles
- GIVEN "leche" 500 ml y "leche" 1 unidad
- WHEN se consolida
- THEN resultan dos líneas separadas

#### Scenario: No aditivo
- GIVEN "sal" al gusto y "sal" 1 pizca
- WHEN se consolida
- THEN la sal no se suma numéricamente y se muestra como "al gusto"/"1 pizca"

### Requirement: Categorización por pasillo
El sistema SHALL asignar una categoría de pasillo (Verdulería, Pescadería, Carnicería, Despensa, Refrigerados, Panadería, Bebidas, Otros), respetando la categoría explícita del ingrediente si existe.

#### Scenario: Heurística por palabra clave
- GIVEN el ingrediente "merluza" sin categoría explícita
- WHEN se categoriza
- THEN se asigna "Pescadería"

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
