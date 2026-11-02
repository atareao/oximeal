# Change: gestión de recetas (edición, borrado y página de recetas)

## Why

La API de recetas solo expone `GET /api/recipes` (listado con búsqueda y
paginación), `GET /api/recipes/{id}` (detalle) y `POST /api/recipes` (alta). No
hay forma de **editar** ni **borrar** una receta: un error tipográfico, un
ingrediente mal introducido o una receta que ya no se cocina quedan atrapados en
la base de datos para siempre. Esto obliga a tocar el SQLite a mano y rompe el
principio de que el usuario gestiona sus propios datos desde la app.

Además, aunque el frontend ya tiene los hooks `useRecipes`/`useRecipe` y las
funciones `fetchRecipes`/`fetchRecipe`, **no existe ninguna página de recetas**:
la navegación solo ofrece Menú y Compra. El usuario no puede ver, buscar ni
seleccionar recetas desde la interfaz, por lo que las funciones de API están
muertas de cara al usuario.

Este change cierra el ciclo CRUD: añade `PUT` y `DELETE` en el backend con sus
validaciones y permisos, y construye una página de recetas completa (listado,
detalle, alta, edición y borrado) integrada en el shell de navegación, con el
mismo cuidado de accesibilidad móvil y comportamiento offline que el resto de la
app. Cierra el issue #45.

## What Changes

- Backend: `PUT /api/recipes/{id}` (permiso `write:recipes`) que reemplaza la
  receta completa —nombre, categoría, `diet_type`, `prep_time_minutes`,
  `servings`, ingredientes, instrucciones y `source_url`—, responde `200` con la
  `Recipe` actualizada, `404` si no existe y `400` si es inválida; el índice FTS
  refleja el cambio vía los triggers `recipes_ai/ad/au`.
- Backend: `DELETE /api/recipes/{id}` (permiso `write:recipes`) que borra la
  receta y sus ingredientes en una transacción `BEGIN IMMEDIATE`, responde `204`
  y `404` si no existe; la receta deja de aparecer en `GET /api/recipes`.
- Ambos endpoints reutilizan la validación de `POST` (categoría y campos) y la
  tabla de permisos de `backend/src/routes/mod.rs`.
- Frontend: nueva página `/recipes` con listado (búsqueda y paginación), detalle,
  alta, edición y borrado con confirmación.
- Frontend: nueva entrada de navegación en `AppLayout` (Sider) y `BottomNav`
  (🍳 Recetas), con estado activo reflejado.
- Accesibilidad: objetivos táctiles ≥48px y textos/roles accesibles en las
  acciones; la UI degrada de forma coherente en modo offline según
  `offline-data`/`offline-writes` (deshabilitar o mostrar estado degradado, sin
  prometer persistencia que no exista).

## Impact

- Capabilities: `recipes` (deltas: actualización y borrado), nueva capacidad
  `recipe-catalog`, y `ui-shell` (delta: pestaña Recetas).
- Backend: `backend/src/routes/recipes.rs`, `backend/src/repo/recipes.rs`,
  `backend/src/models/recipe.rs`, `backend/src/routes/mod.rs`, y nuevos tests de
  integración en `backend/tests/`.
- Frontend: `frontend/src/pages/Recipes.tsx` (nueva) o equivalente,
  `frontend/src/App.tsx`, `frontend/src/components/AppLayout.tsx`,
  `frontend/src/components/BottomNav.tsx`, y tests de componente/hook.
- Sin dependencias nuevas.
- Cierra el issue #45.
