# Comensales habituales y escalado de raciones

## Qué es

OxiMeal guarda una preferencia de **comensales habituales** (1..20, por defecto 2)
y el detalle de cada receta muestra las cantidades escaladas a las raciones
seleccionadas. El valor inicial del selector es la preferencia del hogar.

## Ficheros implicados

### Backend

- `backend/migrations/0010_household_settings.sql` — tabla singleton
  `household_settings` (`id = 1`), `default_servings` con CHECK 1..20 y
  `updated_at`; siembra idempotente con 2.
- `backend/src/models/settings.rs` — `HouseholdSettings` y `SettingsInput`.
- `backend/src/repo/settings.rs` — lectura con fallback a 2 y UPSERT.
- `backend/src/routes/settings.rs` — `GET /api/settings` y `PUT /api/settings`.
- `backend/src/routes/mod.rs` — registro y permisos (`read:recipes` /
  `write:recipes`).
- `backend/src/routes/openapi.rs` — paths y schemas.
- `backend/tests/settings.rs`, `schema_constraints.rs`, `openapi.rs`.

### Frontend

- `frontend/src/lib/scaling.ts` — `scaleAmount`, `formatScaledAmount`,
  `hasRealQuantities`.
- `frontend/src/api/{types,client,hooks}.ts` — tipos, cliente y
  `useSettings`/`useUpdateSettings` (query key `['settings']`).
- `frontend/src/offline/queryPersist.ts` — `settings` persistido en IndexedDB.
- `frontend/src/pages/SettingsPage.tsx` — campo «Comensales habituales».
- `frontend/src/pages/RecipeDetailPage.tsx` — selector único de raciones,
  escalado, aviso y «Ver originales».

## Cómo funciona

1. `GET /api/settings` devuelve `{ default_servings, updated_at }`. Si la fila no
   existe, el repo devuelve 2 sin insertar.
2. `PUT /api/settings` valida 1..20; fuera de rango responde `422` y no modifica
   nada. Sin body, Axum responde `415`.
3. El detalle calcula `factor = raciones_seleccionadas / receta.servings` y
   redondea: resultado ≥10 → entero; ≥1 → un decimal; <1 → dos decimales.
4. Si todas las cantidades son `amount = 1` y `unit = "ud"` (receta importada sin
   parsear), no se escala: se muestran las originales y se avisa.
5. El botón de añadir a la compra envía las raciones seleccionadas; la lista de
   la compra sigue aplicando su propio factor (sin doble escalado).
6. Sin red, `settings` se hidrata desde IndexedDB con el último valor conocido.

## Configuración

No hay variables de entorno nuevas. El rango 1..20 está en el CHECK de la
migración y en la validación de la ruta.

## Decisiones

- **Singleton en BD** en vez de `localStorage`: la preferencia debe compartirse
  entre dispositivos y con clientes M2M.
- **Un único selector** en el detalle: evita dos controles de raciones con
  semánticas distintas.
- **No escalar importadas**: la importación guarda `amount = 1`/`unit = "ud"`;
  escalar produciría cantidades falsas.
- **`updated_at` siempre presente** en el JSON (puede ser `null` si no hay fila).

## Follow-ups

- Parsear cantidades y unidades en la importación para poder escalar recetas
  importadas.
- Unificar la heurística de escalado del backend (lista de la compra) y del
  frontend si aparecen más consumidores.
- Corregir la frase obsoleta de `ui-shell` sobre el cierre de sesión en ajustes.
