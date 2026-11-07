# editable-ingredients

## Qué es
Dos mejoras sobre los ingredientes de las recetas:
1. **Parseo al importar**: al importar una receta por URL se desglosan cantidad, unidad y nombre a partir del texto de cada `recipeIngredient` (antes se guardaba la línea completa como nombre con `1 ud`).
2. **Edición en el detalle**: en el detalle de una receta se editan los ingredientes (nombre, cantidad, unidad) sin abrir el modal completo.

## Ficheros implicados
- `backend/src/domain/ingredient_parse.rs` — helper puro `parse_ingredient()` (+ tests).
- `backend/src/domain/schema_org.rs` — `map_recipe` usa el helper.
- `frontend/src/components/IngredientEditor.tsx` (+ `IngredientEditor.test.tsx`) — editor inline.
- `frontend/src/lib/ingredients.ts` — `parseAmount()` (validación de cantidad).
- `frontend/src/pages/RecipeDetailPage.tsx` / `.css` / `.test.tsx` — integración en el detalle.

## Cómo funciona
- **Import**: `parse_ingredient` reconoce una cantidad inicial (entero, decimal con punto o coma, fracción `a/b` y mixta `a b/c`), una unidad conocida (lista blanca normalizada, plural→singular) y elimina el conector «de». Sin cantidad detectable ⇒ `amount=1`, `unit="ud"`, nombre íntegro. Unidad no reconocida ⇒ se conserva la cantidad y `unit="ud"` (el resto pasa al nombre).
- **Detalle**: el botón «Corregir ingredientes» conmuta la sección a modo edición; valida en cliente (nombre no vacío, cantidad finita ≥ 0) y guarda con `PUT /api/recipes/{id}` reenviando el `RecipeInput` completo (preserva el resto de campos y las etiquetas manuales; `auto_tags: true`). Respeta el escalado de raciones y se deshabilita/degradar sin conexión.

## Configuración y despliegue
No añade variables de entorno ni cambios de API, permisos o esquema de datos (los ingredientes siguen siendo JSON con `name`/`amount`/`unit`). Va con el resto de la app.

## Decisiones
- Parseo en backend (corrige también el consumo del API), no en el frontend.
- Lista blanca de unidades sin conversión de magnitudes (g↔kg), por diseño.
- Los datos ya importados no se re-parsean automáticamente; se corrigen con la edición inline.

## Follow-ups
- Alinear la lista blanca con `units::canonicalize` (tabla única alias→familia→factor→presentación).
- Reparsear recetas ya importadas (migración one-off).
