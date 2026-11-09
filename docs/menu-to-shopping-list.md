# Enlaces a recetas y añadir a la lista desde el menú

## Qué es
En las vistas **diaria** y **semanal** del menú, cada receta planificada enlaza a su ficha (`/recipes/:id`). Junto a la fecha hay un botón **«Añadir a la lista»** que vuelca los ingredientes del día visible (vista diaria) o de la semana completa (vista semanal). En la vista semanal, cada tarjeta de día tiene además su propio botón para añadir solo ese día. El volcado es **aditivo**: fusiona los ingredientes con la lista de la compra existente, conservando los ítems manuales y los ya volcados.

Issue: #123 · Change OpenSpec: `add-menu-to-shopping`.

## Ficheros implicados
- Backend
  - `backend/src/models/shopping_list.rs` — `PlanToShoppingListInput` (`start_date`, `end_date`).
  - `backend/src/repo/shopping_list.rs` — `from_plan`, `merge_drafts`, `validate_range` y `MergeTarget`.
  - `backend/src/routes/shopping_list.rs` — `POST /api/shopping-list/from-plan` en `write_router`.
  - `backend/src/routes/openapi.rs` — registro de la ruta y el esquema.
  - `backend/tests/shopping_list.rs`, `backend/tests/openapi.rs`, `backend/tests/authorization.rs` — tests.
- Frontend
  - `frontend/src/api/types.ts` — `PlanToShoppingListInput`.
  - `frontend/src/api/client.ts` — `addPlanToShoppingList`.
  - `frontend/src/api/hooks.ts` — `useAddPlanToShoppingList` (`networkMode: 'online'`, invalida `shoppingList.all`).
  - `frontend/src/components/DayCard.tsx` — `Link` a la receta y botón por día.
  - `frontend/src/components/WeeklyCalendar.tsx`, `DailyView.tsx` — propagan la acción de añadir.
  - `frontend/src/pages/MenuPage.tsx` — botón de cabecera, mensajes de éxito/error y estado de carga.
  - `frontend/src/pages/MenuPage.test.tsx`, `frontend/src/components/DayCard.test.tsx`, `frontend/src/api/hooks.test.tsx`, `frontend/src/api/client.test.ts` — tests.

## Cómo funciona
- **Enlaces**: `DayCard` renderiza el nombre de la receta como `<Link to={`/recipes/${recipe.id}`}>`.
- **Botones**: 
  - Cabecera de `MenuPage`: visible en `view=day` y `view=week`; `aria-label` «Añadir el día a la lista» / «Añadir la semana a la lista». Se oculta si no hay entradas.
  - `DayCard`: botón «Añadir a la lista» con `aria-label` contextual por fecha; se oculta en la vista diaria para no duplicar la acción de la cabecera.
- **API**: `POST /api/shopping-list/from-plan` con `{ "start_date": "YYYY-MM-DD", "end_date": "YYYY-MM-DD" }`. Ambos obligatorios, `start_date <= end_date`. Permiso `write:shopping`. Respuesta `200` con `[ShoppingListGroup]`.
- **Escalado**: los ingredientes se escalan por las raciones de la entrada del plan y de la receta, igual que en la consolidación.
- **Consolidación**: usa el motor de dominio (`collect_drafts` + `consolidate`) para agrupar y sumar ingredientes.
- **Fusión**: `merge_drafts` busca por `(nombre normalizado, familia, unidad canónica normalizada)`; si coincide, suma la cantidad; si no, inserta una fila nueva. Los ítems existentes conservan `checked`, `client_id`, `source_plan_id` y `source_recipe_id`.
- **Transacción**: `BEGIN IMMEDIATE` para evitar escrituras concurrentes en WAL.
- **Sin cambios de esquema**: los ítems nuevos se insertan con `source_plan_id` y `source_recipe_id` nulos.
- **UI**: estados de carga por botón/fecha, error con `message.error`, deshabilitado sin conexión, invalidación de `shoppingList.all` tras el éxito.

## Variables / configuración
Ninguna nueva. El endpoint usa el permiso existente `write:shopping`. No requiere migraciones ni variables de entorno.

## Cómo habilitar / desplegar
Frontend embebido en el binario: reconstruir la imagen y recrear el contenedor.
```
podman build --format docker -t ghcr.io/atareao/oximeal:local .
podman stop oximeal && podman rm oximeal
podman run -d --name oximeal -p 8080:8080 -v oximeal-data:/app/data --restart unless-stopped -e OIDC_ENABLED=false ghcr.io/atareao/oximeal:local
```

## Decisiones
- **Endpoint nuevo en vez de reutilizar `consolidate`**: `consolidate` reemplaza las líneas derivadas del plan (`source_plan_id IS NOT NULL`), mientras que esta feature exige sumar/fusionar sin borrar. El endpoint nuevo es aditivo y conserva todos los ítems.
- **Clave de fusión por nombre normalizado + unidad canónica**: coherente con el motor de consolidación y con la normalización de unidades. Las unidades `Unknown` se normalizan con `normalize_unit` para que "Cajas" y "cajas" coincidan.
- **Aportes de 0.0** (p. ej. «al gusto») no modifican la fila existente ni su `updated_at`.
- **Redondeo a 2 decimales** al sumar, igual que el motor de consolidación.
- **Validación de rango en el repositorio** además del handler (defensa en profundidad).
- **Sin conexión**: `networkMode: 'online'` evita encolar la mutación; los botones se deshabilitan.

## Follow-ups
- Repetir el volcado suma de nuevo (no es idempotente). La UI deshabilita el botón durante la petición, pero no se ha implementado deduplicación por idempotency key.
- La acción no está disponible en la vista mensual; se puede añadir en el futuro.
- Los tests de `config::tests` fallan en el entorno local por el `.env` presente; no están relacionados con esta feature.