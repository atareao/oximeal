# Change: Salir siempre visible y eliminación de títulos de vista

## Why

Tres hallazgos de UI sobre la versión desplegada:

1. **El botón `Salir` no aparece** cuando la instancia corre en modo abierto
   (`OIDC_ENABLED=false`): `AppHeader` lo renderiza solo con
   `status === 'authenticated'`, de modo que sin sesión no hay ningún punto de
   cierre. Se quiere que la entrada `Salir` esté **siempre** presente en la barra
   superior, a la derecha de Configuración.
2. **La card «Sesión» de Ajustes aparece vacía** fuera del modo autenticado,
   mostrando el texto literal `Sesión` (fallback de `name`), lo que no aporta nada.
3. **Los títulos de página duplican la navegación**: cada vista pinta un
   `<Title level={2}>` visible con «Menú», «Recetas», «Lista de la compra» o
   «Configuración», repetido por la barra superior. Se conserva solo un encabezado
   accesible (oculto visualmente).

## What Changes

- `AppHeader`: renderizar siempre la entrada `Salir`. Con sesión autenticada
  invoca `logout()`; sin sesión autenticada (`disabled`/`anonymous`/`degraded`)
  reinicia la app con una recarga completa (`reloadApp()`), sin llamar a
  `POST /auth/logout`.
- `SettingsPage`: mostrar la sección «Sesión» **solo** con sesión autenticada
  (nombre del usuario). Ocultarla en el resto de estados.
- `MenuPage`, `RecipesPage`, `ShoppingPage`, `SettingsPage`: sustituir el título
  visible por un encabezado de página visualmente oculto (`.visually-hidden`),
  conservando `id` y `aria-labelledby`. Eliminar las reglas CSS de título huérfanas.
- `frontend/src/index.css`: añadir la utilidad `.visually-hidden`.
- `frontend/src/auth/reload.ts` (nuevo): helper `reloadApp()` aislado para poder
  mockearlo en tests.
- Tests: actualizar `AppHeader.test.tsx` (Salir siempre presente; reinicio sin
  sesión), añadir cobertura en `SettingsPage.test.tsx` (sección Sesión oculta sin
  sesión) y ajustar los tests que asumían títulos visibles.

No hay cambios de API ni de backend.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `auth-ui`: «Identidad y cierre de sesión en la interfaz» pasa a exigir `Salir`
  siempre visible (con reinicio si no hay sesión) y la identidad en Ajustes solo
  con sesión autenticada.
- `ui-shell`: «Página de ajustes» se corrige (sin control de logout, identidad
  solo con sesión) y se añade «Encabezados de vista accesibles» (encabezado
  oculto, sin título visible duplicado).

## Impact

- Frontend: `components/AppHeader.tsx`, `pages/SettingsPage.tsx`,
  `pages/MenuPage.tsx`, `pages/RecipesPage.tsx`, `pages/ShoppingPage.tsx`,
  `auth/reload.ts` (nuevo), `index.css` y sus tests.
- Sin cambios de backend, API ni datos.
