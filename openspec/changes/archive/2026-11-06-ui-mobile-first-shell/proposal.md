# Proposal

## Why

La UI actual usa un panel lateral en desktop y una barra inferior en móvil: navegación duplicada, sin acceso a ajustes y sin control de tema. En móvil la barra inferior compite con el contenido y el menú solo ofrece granularidad semanal. Se necesita un shell único mobile-first con navegación superior fija, tema claro/oscuro y vistas diaria, semanal y mensual del menú.

## What Changes

- **BREAKING (UI)**: se elimina el `Layout.Sider` y la `BottomNav`; la navegación pasa a una única barra superior fija.
- Barra superior: logo de OxiMeal a la izquierda; entradas Menú, Recetas y Compra; a la derecha Configuración y Salir. En móvil solo iconos con nombre accesible; desde 768px icono y texto.
- Nueva página `/settings` con selector de tema Oscuro / Claro / Sistema. Por defecto Oscuro; persistencia en `localStorage`.
- Tema aplicado a Ant Design (`darkAlgorithm` / `defaultAlgorithm`) y a variables CSS propias, con `color-scheme` sincronizado y sin destello de tema incorrecto al cargar.
- Vista de menú con selector Diaria / Semanal / Mensual, por defecto Semanal, reflejado en el parámetro `view` de la URL.
- Vista diaria: un día con todas sus franjas y navegación día anterior / siguiente / Hoy.
- Vista mensual: rejilla de calendario del mes con resumen por día; tocar un día abre la vista diaria.
- La entrada Asistente desaparece de la navegación; el acceso a Valet se conserva desde las acciones del menú.
- Tests de UI actualizados y nuevos para shell, tema y vistas.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `ui-shell`: navegación superior única, eliminación de Sider y BottomNav, tema claro/oscuro/sistema y página de ajustes.
- `weekly-menu`: selector de vista diaria/semanal/mensual, vista diaria y rejilla mensual.
- `auth-ui`: el cierre de sesión se traslada a la barra superior y a la página de ajustes.

## Impact

- Frontend: `AppLayout`, `AppHeader` (nuevo), `BottomNav` (eliminado), `theme.ts`, `ThemeProvider` (nuevo), `SettingsPage` (nuevo), `MenuPage`, `DailyView` y `MonthlyCalendar` (nuevos), `App.tsx`, `index.css`.
- Tests: `AppLayout.test.tsx`, `BottomNav.test.tsx` (eliminado), `MenuPage.test.tsx`, y nuevos `AppHeader.test.tsx`, `ThemeProvider.test.tsx`, `SettingsPage.test.tsx`.
- Sin cambios de backend, API ni base de datos. Se reutiliza `GET /api/meal-plan?start_date&end_date`.
- Dependencias: ninguna nueva (antd v6 ya incluye los algoritmos de tema).
