# Tasks — Salir siempre visible y títulos de vista ocultos (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Leer `frontend/src/components/AppHeader.tsx`, `frontend/src/components/AppHeader.test.tsx`, `frontend/src/pages/SettingsPage.tsx`, `frontend/src/pages/SettingsPage.test.tsx` y las cuatro páginas de vista.
- [x] 1.2 Crear `frontend/src/auth/reload.ts` con `reloadApp()` (helper mockeable) y la utilidad `.visually-hidden` en `frontend/src/index.css`.

## 2. RED

- [x] 2.1 `AppHeader.test.tsx`: invertir «sin sesión autenticada no renderiza Salir» → `Salir` presente en los estados `loading`, `disabled`, `anonymous`, `degraded`.
- [x] 2.2 `AppHeader.test.tsx`: nuevo test — pulsar `Salir` sin sesión llama a `reloadApp()` y NO al `logout`.
- [x] 2.3 `SettingsPage.test.tsx`: nuevo test — con `status: 'disabled'` no se muestra la sección «Sesión».
- [x] 2.4 Tests de páginas: asegurar que existe el encabezado accesible oculto y que NO hay título visible.
- [x] 2.5 Ejecutar `npx vitest run` y confirmar RED.

## 3. GREEN

- [x] 3.1 `AppHeader.tsx`: quitar la condición `status === 'authenticated'`; `handleLogout` usa `logout()` si autenticado y `reloadApp()` en el resto.
- [x] 3.2 `SettingsPage.tsx`: renderizar la sección «Sesión» solo si `status === 'authenticated'`.
- [x] 3.3 Páginas: sustituir `<Title ... level={2} className="*__title">` por el mismo `Title` con `className="visually-hidden"` (conservar `id`), y eliminar las reglas CSS `*__title` huérfanas.
- [x] 3.4 `npx vitest run` y `npx tsc --noEmit` en verde.

## 4. REFACTOR

- [x] 4.1 `npm run lint` y `npx prettier --check` sin novedades.
- [x] 4.2 Re-ejecutar `npx vitest run` (sin regresiones).

## 5. Cierre

- [x] 5.1 `openspec validate logout-titles-cleanup --strict` en verde.
- [x] 5.2 Actualizar `CHANGELOG.md` (sección Corregido).
- [x] 5.3 `just ci-ui` en verde; PR a `development` y archivar el change.
