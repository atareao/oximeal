## ADDED Requirements

### Requirement: Alta de receta
El sistema SHALL exponer `POST /api/agent/recipes` para que un cliente M2M con el permiso `agent:write` cree una receta. El cuerpo SHALL ser el mismo `RecipeInput` que `POST /api/recipes` y SHALL aplicar la misma validación (nombre no vacío, categoría válida, al menos un ingrediente, cantidades no negativas). La respuesta SHALL ser `201` con la receta creada.

#### Scenario: Alta válida
- GIVEN un token M2M con `agent:write`
- WHEN se envía un `RecipeInput` válido
- THEN se crea la receta y se devuelve `201` con su `id`

#### Scenario: Receta inválida
- GIVEN un token M2M con `agent:write`
- WHEN el cuerpo no tiene ingredientes o la categoría no es válida
- THEN se devuelve `400` y no se crea nada

#### Scenario: Sin permiso de escritura
- GIVEN un token M2M con `agent:read` pero sin `agent:write`
- WHEN se llama a `POST /api/agent/recipes`
- THEN se devuelve `403`