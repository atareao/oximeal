# Change: cierre de sesión único en la barra superior

## Why

El cierre de sesión está **duplicado**: la barra superior (`AppHeader`) ya ofrece
el control `Salir` (visible solo con sesión autenticada), pero `SettingsPage`
mantiene un bloque «Cuenta» con un segundo botón `Salir`. Dos controles para la
misma acción confunden y obligan a mantener el doble de código y pruebas. Se
unifica el cierre de sesión en un único sitio: la barra superior, a la derecha de
Configuración.

## What Changes

- **Eliminar** el bloque «Cuenta» de `SettingsPage` (título, `settings-page__logout`,
  `handleLogout` y el uso de `logout` de `useSession`).
- **Conservar** la identidad del usuario (su nombre) en la página de ajustes, que
  sigue siendo su sitio natural.
- **No tocar** el control `Salir` de `AppHeader` (`app-header__logout`), que pasa a
  ser el único punto de cierre de sesión.
- Retirar el test «el botón Salir invoca el logout de la sesión» de
  `SettingsPage.test.tsx` y cubrir el nuevo comportamiento (ajustes sin logout,
  nombre visible).
- No hay cambios de API ni de backend: `POST /auth/logout` se invoca igual que hoy.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `auth-ui`: el requirement «Identidad y cierre de sesión en la interfaz» pasa de
  ofrecer el cierre de sesión en la barra superior **y** en ajustes a ofrecerlo
  **solo** en la barra superior, manteniendo la identidad del usuario en ajustes.

## Impact

- Frontend: `frontend/src/pages/SettingsPage.tsx` (elimina el bloque de logout) y
  `frontend/src/pages/SettingsPage.test.tsx` (retira el test de logout).
- Spec: `openspec/specs/auth-ui/spec.md` (MODIFIED del requirement citado).
- Sin cambios en backend, API, dependencias ni estilo de `AppHeader`.
