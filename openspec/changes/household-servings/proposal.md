# Change: comensales habituales y escalado en el detalle

## Why

Hoy no existe ninguna preferencia de comensales y el detalle de una receta
siempre muestra las cantidades **originales**. Una receta importada para 8 se
sigue leyendo como para 8 aunque la casa sea de 2; el único escalado existente
vive en la lista de la compra (`factor = raciones_pedidas / raciones_receta`).
El usuario pide (1) configurar los comensales habituales y (2) saber qué pasa al
abrir una receta de 8 con preferencia 2.

Además, la importación guarda los ingredientes sin parsear (`amount = 1`,
`unit = "ud"`), así que escalar esos valores produciría cantidades absurdas
(0,25 ud). El diseño debe detectarlo y avisar en vez de mentir.

## What Changes

- **Migración `0010`**: tabla singleton `household_settings` (`id = 1`) con
  `default_servings INTEGER NOT NULL DEFAULT 2 CHECK (BETWEEN 1 AND 20)` y
  `updated_at`. Fila sembrada de forma idempotente.
- **API**: `GET /api/settings` devuelve `{ "default_servings": n }` con permiso
  `read:recipes`; `PUT /api/settings` con `write:recipes` valida 1..20 y
  responde `422` fuera de rango. OpenAPI y tabla de permisos actualizados.
- **Ajustes**: campo numérico «Comensales habituales» (1..20) con guardado
  explícito, error visible y valor por defecto 2 si nunca se ha guardado.
- **Detalle de receta**: selector de raciones (1..20) que por defecto toma la
  preferencia; las cantidades se escalan por `raciones_seleccionadas /
  raciones_receta`; aviso «Receta para 8 · cantidades para 2» y opción de ver las
  originales. Si la receta viene de importación sin cantidades parseadas, se
  muestran las originales con aviso y **no** se escala.
- **Lista de la compra**: sin cambios en su cálculo; el selector del detalle solo
  fija el valor inicial al añadir desde la receta.
- **Offline**: la consulta de ajustes se persiste en IndexedDB; sin red se usa el
  último valor conocido.

## Capabilities

### New Capabilities

- `household-settings`: preferencia de comensales habituales, persistida en el
  servidor y expuesta por API.

### Modified Capabilities

- `recipes`: «Detalle de receta» incorpora el escalado de cantidades con selector,
  valor por defecto de la preferencia, aviso de origen y vista de originales.

## Impact

- Backend: `migrations/0010_household_settings.sql`, `models/settings.rs`,
  `repo/settings.rs`, `routes/settings.rs`, `routes/mod.rs`, `routes/openapi.rs`,
  `lib.rs`/`main.rs` si hace falta módulo, tests `tests/settings.rs`.
- Frontend: `api/types.ts`, `api/client.ts`, `hooks.ts`, `pages/SettingsPage.tsx`
  (+CSS), `pages/RecipeDetailPage.tsx` (+CSS), `lib/scaling.ts`, tests.
- Specs: nueva `household-settings`; MODIFIED de `recipes`. (El campo de ajustes
  se especifica en `household-settings`; `ui-shell` no se toca.)
- Sin cambios en `ingredient-consolidation` ni en el esquema de recetas.
