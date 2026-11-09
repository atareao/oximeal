# shopping-list Specification (delta)

## ADDED Requirements

### Requirement: Deduplicación de líneas manuales con la misma clave
Al arrancar, el sistema SHALL reconciliar una única vez, con marca persistente, las líneas manuales de `shopping_list_items` (sin `source_plan_id` ni `source_recipe_id`) que compartan la misma clave: nombre normalizado y unidad canónica (familia y presentación). SHALL sumar sus cantidades (2 decimales), conservar la fila de `id` menor, marcar `checked` si alguna lo estaba y eliminar el resto. Orígenes distintos NO SHALL fusionarse.

#### Scenario: Fusión de duplicados heredados
- GIVEN dos líneas manuales con la misma clave («Tomate» 2 ud y «Tomate» 3 ud)
- WHEN se ejecuta la reconciliación
- THEN queda una única línea «Tomate» de 5 ud

#### Scenario: Unidades compatibles se canonicalizan
- GIVEN dos líneas manuales «Harina» de «1 kg» y «500 g»
- WHEN se ejecuta la reconciliación
- THEN queda una única línea equivalente a 1500 g

#### Scenario: Unidades incompatibles no se fusionan
- GIVEN dos líneas manuales «Tomate» de «2 ud» y «500 g»
- WHEN se ejecuta la reconciliación
- THEN siguen siendo dos líneas

#### Scenario: Orígenes distintos no se fusionan
- GIVEN una línea manual «Tomate» de 2 ud y una línea del plan «Tomate» de 4 ud
- WHEN se ejecuta la reconciliación
- THEN siguen siendo dos líneas

#### Scenario: Reconciliación idempotente
- GIVEN una base ya reconciliada
- WHEN el servicio se vuelve a ejecutar
- THEN no cambia ninguna línea
- AND no se repite el trabajo (marca persistente)

## MODIFIED Requirements

### Requirement: Lista agrupada
El sistema SHALL exponer `GET /api/shopping-list` devolviendo los ítems agrupados por categoría de pasillo y con su estado `checked`. El listado SHALL ordenarse de forma determinista por nombre sin distinguir mayúsculas (`name COLLATE NOCASE`) y, a igualdad de nombre, por `id` ascendente.

#### Scenario: Agrupación
- GIVEN ítems en varias categorías
- WHEN se consulta la lista
- THEN la respuesta se agrupa por categoría con los ítems dentro

#### Scenario: Orden determinista
- GIVEN dos ítems con el mismo nombre y distinta capitalización
- WHEN se consulta la lista repetidamente
- THEN aparecen siempre en el mismo orden dentro de su categoría
