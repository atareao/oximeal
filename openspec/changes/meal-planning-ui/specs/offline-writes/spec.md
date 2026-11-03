# Capability: offline-writes

## ADDED Requirements

### Requirement: Planificación de comidas requiere conexión

Las mutaciones de la planificación de comidas (`POST`, `PATCH` y `DELETE` sobre
`/api/meal-plan`) SHALL requerir conexión y SHALL **no** encolarse en IndexedDB,
a diferencia de las escrituras de la lista de la compra. Sin conexión la
aplicación SHALL deshabilitar los controles de planificación y SHALL mostrar un
aviso de modo degradado, sin aplicar cambios optimistas que prometan una
persistencia inexistente.

#### Scenario: Planificación sin conexión
- GIVEN la vista semanal abierta sin conexión
- WHEN el usuario intenta añadir, editar raciones o quitar una entrada
- THEN los controles de planificación están deshabilitados
- AND se muestra un aviso de que la planificación requiere conexión
- AND no se encola ninguna operación en IndexedDB

#### Scenario: Recuperación de la conexión
- GIVEN la vista semanal en modo degradado sin conexión
- WHEN se recupera la conexión
- THEN los controles de planificación se habilitan de nuevo
- AND el plan se refresca desde el servidor
