# Capability: offline-writes

## ADDED Requirements

### Requirement: Operaciones compuestas no encoladas

Las operaciones de reconstrucción de la lista que agregan o reconsolidan
múltiples ítems y dependen del servidor (`POST /api/shopping-list/consolidate` y
`POST /api/shopping-list/from-recipe`) NO SHALL encolarse en IndexedDB. Sin
conexión, la UI SHALL deshabilitarlas o mostrarlas en estado degradado,
reservando la cola de escrituras para las mutaciones por ítem (alta, toggle y
edición de cantidad).

#### Scenario: Sin encolado offline

- GIVEN la lista de la compra abierta sin conexión
- WHEN el usuario intenta consolidar desde el menú o volcar una receta
- THEN la acción está deshabilitada y no se encola ninguna operación
- AND no se promete una persistencia inexistente

#### Scenario: La cola por ítem sigue operativa

- GIVEN una operación compuesta deshabilitada sin conexión
- WHEN el usuario añade o marca un ítem manualmente
- THEN la mutación por ítem se encola y se aplica de forma optimista como hoy
