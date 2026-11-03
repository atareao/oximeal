# Capability: ui-shell

## MODIFIED Requirements

### Requirement: Objetivos táctiles

En móvil, las filas y controles interactivos SHALL tener una altura mínima de
48px para garantizar una interacción táctil cómoda. Esta regla SHALL aplicar
también a los controles de planificación del menú semanal (añadir receta, quitar
entrada, editar raciones y selector de recetas).

#### Scenario: Control interactivo en móvil
- GIVEN la vista de lista de la compra en un viewport móvil
- WHEN se inspecciona una fila o control interactivo
- THEN su altura es de al menos 48px

#### Scenario: Controles de planificación en móvil
- GIVEN la vista de menú semanal en un viewport móvil
- WHEN se inspeccionan los controles de añadir receta, quitar entrada y editar raciones
- THEN su altura es de al menos 48px
