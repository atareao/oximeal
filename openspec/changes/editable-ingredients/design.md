# Design

## Context

Los ingredientes se guardan como JSON embebido en `recipes.ingredients` con `name`, `amount` (f64), `unit` (String libre) y `category` opcional. El import mapea hoy cada `recipeIngredient` sin parsear (nombre completo, `amount = 1`, `unit = "ud"`). La edición ya existe en el modal `RecipeForm`; el detalle es de solo lectura.

## Goals / Non-Goals

**Goals:**
- Desglosar cantidad y unidad al importar sin perder información.
- Editar ingredientes en el detalle sin abrir el modal.

**Non-Goals:**
- Catálogo/enum de unidades ni conversión de magnitudes (g↔kg).
- Reescribir el modal `RecipeForm` o el esquema SQLite.

## Decisions

- **Parseo en backend, en un helper puro y testeable** (módulo nuevo, p. ej. `backend/src/domain/ingredient_parse.rs`), invocado desde `schema_org::map_recipe`. Alternativa descartada: parsear en el frontend (no corrige datos ya persistidos ni el consumo del API).
- **Reglas de parseo**: token de cantidad inicial aceptando entero, decimal con punto o coma y fracción `a/b` (y mixta `a b/c`); si el token siguiente es una unidad conocida se toma como `unit`; se elimina el conector «de» tras la unidad; `name` es el resto en su orden original. Sin cantidad detectable ⇒ `amount = 1`, `unit = "ud"`, `name` íntegro.
- **Unidades conocidas**: lista blanca normalizada (`g`, `kg`, `ml`, `l`, `cl`, `ud`, `uds`, `taza`, `cucharada`, `cucharadita`, `pizca`, ...). Unidad no reconocida ⇒ se conserva la cantidad y se usa `unit = "ud"` manteniendo el resto como nombre (evita tratar palabras como unidad por error).
- **Edición inline en el detalle**: filas editables con guardado por ingrediente reutilizando `useUpdateRecipe` (mismo `PUT`), manteniendo el resto del payload intacto. Se refleja de inmediato en la lista y en el escalado (que ya depende de cantidades reales).

## Risks / Trade-offs

- **Falsos positivos/negativos** en texto libre: se mitiga con lista blanca de unidades y casos de test; ante duda se conserva el texto íntegro (nunca se pierde información).
- **Datos ya importados** no se re-parsean automáticamente; la edición inline permite corregirlos (follow-up posible: reparsear).
- **Concurrencia**: cada guardado inline reemplaza la receta completa (`PUT`); gana la última edición. Aceptable con usuario único por hogar.
