# Spec Delta

## MODIFIED Requirements

### Requirement: Alta rápida y limpieza
La vista SHALL ofrecer un input superior para añadir ítems manualmente y, en la barra de acciones, dos acciones: «Limpiar comprados» (invoca `DELETE /api/shopping-list/completed`) y «Marcar todo», que marca como comprados todos los ítems pendientes reutilizando el toggle por ítem (por lo que se encola sin conexión). «Limpiar comprados» SHALL estar deshabilitado sin ítems comprados y «Marcar todo» sin ítems pendientes.

#### Scenario: Alta manual
- GIVEN el modo compra abierto
- WHEN el usuario escribe un ítem en el input superior y confirma
- THEN el ítem se crea y aparece bajo el pasillo que le corresponde

#### Scenario: Limpiar comprados
- GIVEN una lista con ítems marcados como comprados
- WHEN el usuario pulsa "Limpiar comprados"
- THEN se invoca `DELETE /api/shopping-list/completed`
- AND los ítems comprados desaparecen de la vista

#### Scenario: Marcar todo
- GIVEN una lista con varios ítems pendientes
- WHEN el usuario pulsa "Marcar todo"
- THEN todos los ítems pendientes quedan marcados como comprados
- AND los ítems ya marcados no cambian

#### Scenario: Marcar todo sin conexión
- GIVEN la lista con ítems pendientes y sin conexión
- WHEN el usuario pulsa "Marcar todo"
- THEN los ítems quedan marcados de forma optimista y el cambio se encola para sincronizar

#### Scenario: Marcar todo deshabilitado sin pendientes
- GIVEN una lista cuyos ítems están todos marcados
- WHEN se muestra la barra de acciones
- THEN "Marcar todo" aparece deshabilitado
