## MODIFIED Requirements

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

## ADDED Requirements

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