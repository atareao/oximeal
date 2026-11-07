--- INICIO CONTENIDO ---
# PLAN-002 — Salir siempre visible y encabezados de vista ocultos

> Plan generado por el Project Director. Change OpenSpec asociado: `logout-titles-cleanup`.
> Rama: `feature/logout-titles-cleanup` → PR a `development`.

## 1. Objetivo

Cerrar tres hallazgos de UI reportados sobre la app desplegada (OxiMeal, frontend React 19 + Antd v6):

1. **`Salir` ausente en la barra**: solo se renderiza con `status === 'authenticated'`; en modo abierto (`OIDC_ENABLED=false`) no hay ningún punto de cierre. Se quiere la entrada `Salir` **siempre** visible, a la derecha de Configuración.
2. **Card «Sesión» vacía en Ajustes**: `SettingsPage` la pinta siempre y, sin sesión, muestra el literal `Sesión`. Debe mostrarse solo con sesión autenticada.
3. **Títulos de vista redundantes**: cada vista pinta un `<Title level={2}>` visible («Menú», «Recetas», «Lista de la compra», «Configuración») que duplica la navegación. Se elimina el título visible y se conserva solo un encabezado accesible oculto.

## 2. Decisiones de diseño

- **Semántica de `Salir` sin sesión**: con sesión autenticada invoca `logout()`; en `disabled`/`anonymous`/`degraded` reinicia la app con una recarga completa (`reloadApp()`), sin llamar a `POST /auth/logout` (en modo abierto la ruta no existe).
- **`reloadApp()` aislado** en `frontend/src/auth/reload.ts` para poder mockearlo en tests (jsdom no implementa `window.location.reload`).
- **Encabezado oculto**: se conserva `<Title id=... level={2}>` con `className="visually-hidden"`; el `id` y el `aria-labelledby` de la sección se mantienen para lectores de pantalla. Se eliminan las reglas CSS `*__title` que quedan huérfanas.
- **Utilidad `.visually-hidden`** nueva en `frontend/src/index.css`.
- **Card «Ses.»**: se mantiene la identidad (nombre) solo con `status === 'authenticated'`, según el requisito `auth-ui`.

## 3. Alcance (ficheros)

- `frontend/src/components/AppHeader.tsx` + `AppHeader.test.tsx`
- `frontend/src/pages/SettingsPage.tsx` + `SettingsPage.test.tsx`
- `frontend/src/pages/MenuPage.tsx`, `RecipesPage.tsx`, `ShoppingPage.tsx` (+ CSS de página)
- `frontend/src/auth/reload.ts` (nuevo)
- `frontend/src/index.css`

Sin cambios de backend, API ni datos.

## 4. Tareas (TDD RED → GREEN → REFACTOR)

1. Helper `reloadApp()` + utilidad `.visually-hidden`.
2. RED: tests de `AppHeader` (Salir siempre presente; reinicio sin sesión), de `SettingsPage` (sección Sesión oculta sin sesión) y de páginas (encabezado oculto / sin título visible).
3. GREEN: `AppHeader`, `SettingsPage`, las cuatro páginas y limpieza de CSS.
4. REFACTOR: `lint`, `prettier --check`, `vitest run`, `tsc --noEmit`.
5. Cierre: `openspec validate logout-titles-cleanup --strict`, `CHANGELOG.md`, `just ci-ui`, PR a `development`.

## 5. Criterios de aceptación (DoD)

- [ ] `Salir` visible a la derecha de Configuración en todos los estados de sesión conocidos.
- [ ] Con sesión autenticada `Salir` cierra sesión; sin sesión recarga la app y no llama al logout.
- [ ] La sección «Sesión» de Ajustes solo aparece con sesión autenticada.
- [ ] Ninguna vista muestra un título de página visible; existe encabezado accesible oculto con `aria-labelledby` válido.
- [ ] `npx vitest run`, `npx tsc --noEmit`, `npm run lint`, `just ci-ui` y `openspec validate --all --strict` en verde.
- [ ] Change `logout-titles-cleanup` archivado y `CHANGELOG.md` actualizado.

## 6. Riesgos

| Riesgo | Mitigación |
|---|---|
| jsdom no implementa `window.location.reload` | `reloadApp()` en módulo aislado, mockeado con `vi.mock`.
| Tests que asumían títulos visibles | Revisar y actualizar; `getAllByRole('heading', {level:2})` de `ShoppingPage.test.tsx` sigue válido al conservar `level={2}`.
| Jerarquía de encabezados | Se conserva `level={2}` para no alterar la jerarquía actual (pasillos y secciones son h2/h3).
--- FIN CONTENIDO ---