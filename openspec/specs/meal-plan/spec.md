# meal-plan Specification

## Purpose
Esta capacidad gestiona la planificación semanal de comidas. Permite consultar el plan por rango de fechas, asignar recetas a franjas horarias (slots) y eliminar asignaciones existentes.

## Requirements

### Requirement: Consulta de plan por rango
El sistema SHALL exponer `GET /api/meal-plan?start_date=&end_date=` devolviendo las asignaciones del rango con su receta.

#### Scenario: Rango válido
- GIVEN asignaciones en un rango de fechas
- WHEN se consulta el rango
- THEN se devuelven las entradas ordenadas por fecha y tipo de comida

### Requirement: Asignación de slot
El sistema SHALL exponer `POST /api/meal-plan` validando `meal_type` (lunch|dinner|breakfast|snack) y `role` (main|salad|side) y referenciando una receta existente.

#### Scenario: Receta inexistente
- GIVEN un `recipe_id` que no existe
- WHEN se envía la asignación
- THEN se devuelve error de validación y no se persiste

### Requirement: Eliminación de slot
El sistema SHALL exponer `DELETE /api/meal-plan/{id}`.

#### Scenario: Eliminación correcta
- GIVEN una entrada existente
- WHEN se elimina por id
- THEN se devuelve 204 y deja de aparecer en el rango
