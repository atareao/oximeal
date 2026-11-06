# Tasks — Cierre de sesión único en la barra superior (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Leer `frontend/src/pages/SettingsPage.tsx`, `frontend/src/pages/SettingsPage.test.tsx` y `frontend/src/components/AppHeader.tsx` para confirmar el bloque «Cuenta» y el control `Salir` de la barra (verificación: lectura hecha, sin cambios)
- [x] 1.2 Confirmar que `AppHeader` ya muestra `Salir` solo con `status === 'authenticated'` y no requiere cambios (verificación: `rg -n 'app-header__logout|status ===' frontend/src/components/AppHeader.tsx`)

## 2. RED

- [x] 2.1 Ajustar `SettingsPage.test.tsx`: retirar el test «el botón Salir invoca el logout de la sesión» (verificación: `npx vitest run frontend/src/pages/SettingsPage.test.tsx` falla por el test que aún afirma presencia del logout)
- [x] 2.2 Añadir test `SettingsPage.test.tsx`: la página de ajustes NO muestra ningún control con `aria-label="Salir"` ni `settings-page__logout` (verificación: el test falla contra la implementación actual)
- [x] 2.3 Añadir test `SettingsPage.test.tsx`: el nombre del usuario sigue visible en ajustes (verificación: el test pasa ya que la identidad se conserva)
- [x] 2.4 Ejecutar `npx vitest run` y confirmar el estado RED (verificación: fallan los tests del punto 2.1/2.2)

## 3. GREEN

- [x] 3.1 En `SettingsPage.tsx`, eliminar el bloque «Cuenta» completo: título, `settings-page__identity`, botón `settings-page__logout`, `handleLogout` y el `logout` obtenido de `useSession` (verificación: `rg -n 'logout|settings-page__logout|Cuenta' frontend/src/pages/SettingsPage.tsx` no devuelve nada)
- [x] 3.2 Mantener la lectura de `name` desde `useSession` y el render del nombre para conservar la identidad en ajustes (verificación: test 2.3 en verde)
- [x] 3.3 Ejecutar `npx vitest run` (verificación: todos los tests en verde) y `npx tsc --noEmit` (verificación: sin errores de tipos)

## 4. REFACTOR

- [x] 4.1 Revisar que `SettingsPage.tsx` no deja imports ni clases CSS huérfanas del bloque eliminado (verificación: `npx tsc --noEmit` sin errores y sin referencias a `settings-page__logout`/`settings-page__identity` en el fuente)
- [x] 4.2 `npm run lint` (verificación: sin warnings nuevos)
- [x] 4.3 Re-ejecutar `npx vitest run` para descartar regresiones (verificación: 100% verde)

## 5. Cierre

- [x] 5.1 Actualizar el spec con `openspec archive settings-logout` una vez aprobado y validado (verificación: `openspec validate settings-logout --strict` en verde antes de archivar)
