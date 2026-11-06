# Tareas — ui-mobile-first-shell

## 1. RED

- [x] 1.1 `AppHeader.test.tsx`: barra superior con logo, Menú, Recetas, Compra, Configuración y Salir; sin Sider ni BottomNav.
- [x] 1.2 `AppHeader.test.tsx`: en móvil solo iconos con nombre accesible; en desktop icono y texto; estado activo con `aria-current`.
- [x] 1.3 `ThemeProvider.test.tsx`: tema oscuro por defecto, persistencia en `localStorage`, modo Sistema con `matchMedia`.
- [x] 1.4 `SettingsPage.test.tsx`: ruta `/settings`, selector de tema, nombre de usuario y Salir.
- [x] 1.5 `MenuPage.test.tsx`: vista semanal por defecto, cambio a diaria y mensual, `view` en la URL, navegación de día y mes, rejilla mensual.
- [x] 1.6 Ejecutar `npx vitest run`; confirmar rojo por las causas esperadas.

## 2. GREEN

- [x] 2.1 `ThemeProvider` + variables CSS de tema en `index.css` + script anti-destello en `index.html`.
- [x] 2.2 `AppHeader` con logo, navegación y acciones; `AppLayout` usa solo la barra superior.
- [x] 2.3 Eliminar `BottomNav` y sus estilos; actualizar `AppLayout.test.tsx`.
- [x] 2.4 Ruta `/settings` y `SettingsPage` con selector de tema e identidad.
- [x] 2.5 Selector de vista en `MenuPage` con estado en la URL.
- [x] 2.6 `DailyView` reutilizando `DayCard` y navegación de días.
- [x] 2.7 `MonthlyCalendar` con rejilla, resumen por día y navegación de meses.
- [x] 2.8 `npx vitest run` y `npx tsc --noEmit` en verde.

## 3. REFACTOR

- [x] 3.1 Sustituir hex hardcodeados de los `.css` por variables de tema.
- [x] 3.2 Revisar accesibilidad: `aria-label`, `aria-current`, foco y objetivos táctiles de 48px.
- [x] 3.3 `npm run lint`, `npm run format:check` y `just ci-ui`.

## 4. Documentación y cierre

- [x] 4.1 Actualizar `CHANGELOG.md` en `[No publicado]`.
- [x] 4.2 `openspec validate --all --strict`.
- [x] 4.3 Archivar el change.
- [x] 4.4 PR a `development` con CI en verde.
