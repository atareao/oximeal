# Design

## Context

El frontend es React 19 + TypeScript + Vite, con Ant Design v6, `react-router-dom` v7 y `@tanstack/react-query` v5. El shell actual (`src/components/AppLayout.tsx`) usa `useMediaQuery('(min-width: 768px)')` para mostrar un `Layout.Sider` en desktop y una `BottomNav` en móvil. Los colores están repartidos entre `src/components/theme.ts` (tokens antd) y varios `.css` con hex hardcodeados; no hay modo oscuro ni variables CSS de tema. El menú (`src/pages/MenuPage.tsx`) solo soporta la semana mediante `?week=YYYY-MM-DD`.

## Goals

- Una sola barra de navegación superior, fija y mobile-first.
- Tema Oscuro por defecto, con Claro y Sistema, persistido y sin destello.
- Tres vistas de menú (diaria, semanal, mensual) con Semanal por defecto.
- Mantener el comportamiento existente de planificación, offline y accesibilidad táctil.

## Non-Goals

- No se cambia el backend ni el contrato de `GET /api/meal-plan`.
- No se añade chat nativo ni se modifica la integración con Valet.
- No se rediseñan las páginas de Recetas, Compra ni el detalle de receta.

## Decisions

### 1. Barra superior única (`AppHeader`)

Se sustituyen `Layout.Sider` y `BottomNav` por un `AppHeader` fijo (`position: sticky; top: 0`) con tres zonas: logo, navegación principal y acciones. La navegación usa `NavLink` para obtener `aria-current="page"`. En viewports menores de 768px las entradas muestran solo icono con `aria-label` y `title`; desde 768px muestran icono y texto. Todos los controles tienen una altura mínima de 48px. Se aplica `padding-top: env(safe-area-inset-top)`.

La entrada Asistente desaparece. El acceso a Valet se mantiene en las acciones de `MenuPage` (estado vacío y acción de planificar con Valet), que ya existe.

### 2. Tema con `ThemeProvider`

Nuevo `src/theme/ThemeProvider.tsx` con contexto `{ mode, resolvedMode, setMode }`. `mode` es `dark | light | system`; `resolvedMode` es `dark | light`. La preferencia se guarda en `localStorage` bajo `oximeal.theme`. En modo `system` se usa `matchMedia('(prefers-color-scheme: dark)')` y se escucha su evento `change`.

Para evitar el destello, un script inline mínimo en `index.html` lee `localStorage` y fija `data-theme` y `color-scheme` en `<html>` antes de cargar el bundle. `ThemeProvider` sincroniza después el estado de React con ese valor.

`ConfigProvider` recibe `theme={{ algorithm: resolvedMode === 'dark' ? theme.darkAlgorithm : theme.defaultAlgorithm, token: ... }}`. Los colores propios se exponen como variables CSS en `:root[data-theme='dark']` y `:root[data-theme='light']` en `src/index.css`; los `.css` de componentes dejan de usar hex fijos y consumen esas variables.

### 3. Estado de la vista de menú en la URL

`MenuPage` lee `view` (`day | week | month`, por defecto `week`) y la fecha de referencia (`date` o `week`, según la vista) de `useSearchParams`. Cambiar de vista actualiza la URL, de modo que recargar o compartir conserva el estado. La vista semanal mantiene `?week=YYYY-MM-DD` por compatibilidad; la diaria usa `?view=day&date=YYYY-MM-DD`; la mensual usa `?view=month&date=YYYY-MM-01`.

### 4. Vistas diaria y mensual

- **Diaria**: reutiliza `DayCard` para un único día y las mismas acciones de planificación. Navegación anterior / siguiente / Hoy.
- **Mensual**: nuevo `MonthlyCalendar` con rejilla de 7 columnas y celdas de al menos 44px. Cada celda muestra el número de día y un resumen compacto (número de comidas y, si cabe, el nombre de la comida principal). Tocar una celda navega a `?view=day&date=...`.
- Ambas vistas consumen `useMealPlan` con el rango visible: un día, la semana de lunes a domingo o el mes completo.

### 5. Página de ajustes

Nueva ruta `/settings` con `SettingsPage`: selector de tema (`Segmented` o `Radio.Group`) con las tres opciones y bloque de identidad con nombre de usuario y botón Salir. La entrada Configuración de la barra superior navega a esta ruta.

## Risks / Trade-offs

- **Eliminar la BottomNav** reduce el acceso directo a Valet; se mitiga manteniendo las acciones de Valet en el menú y documentándolo en la spec.
- **Destello de tema**: requiere el script inline en `index.html`; si se olvida, el primer frame sale claro.
- **Colores hardcodeados**: hay que revisar todos los `.css` para que el modo claro/oscuro sea coherente; se hará por barrido con `rg` de hex.
- **Tests existentes**: `AppLayout.test.tsx` y `BottomNav.test.tsx` asumen la estructura anterior y deben reescribirse o eliminarse.

## Migration Plan

1. Introducir `ThemeProvider` y las variables CSS sin cambiar la navegación; verificar que el tema oscuro se aplica.
2. Sustituir `AppLayout` por `AppHeader` y eliminar `BottomNav`; actualizar tests.
3. Añadir `/settings` y el selector de tema.
4. Añadir el selector de vista y las vistas diaria y mensual.
5. Ejecutar `just ci-ui` y validar OpenSpec.

## Open Questions

- Ninguna bloqueante. El icono concreto de cada entrada se decide en implementación (se usan iconos de antd, sin dependencia nueva).
