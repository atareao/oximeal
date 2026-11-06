# Change: Configuración y Salir siempre visibles en la barra superior

## Why

En la barra superior (`AppHeader`) el control `Salir` es el **último** elemento y
en viewports estrechos queda fuera de pantalla. El ancho mínimo real es de unos
440px: logo (~106px) + 6 controles de 48px (288px) + huecos (24px) + padding
(24px). Por debajo de ese ancho (360–430px, mayoría de móviles) el header no
recorta ni hace scroll: simplemente desborda el viewport y `Salir` deja de ser
visible y pulsable. El usuario reporta no verlo junto a Configuración.

Además, la spec `ui-shell` afirma que la barra incluye «Configuración y Salir»
sin matizar que `Salir` sólo existe con sesión autenticada (en modo
`disabled`, con OIDC apagado, no hay sesión ni logout). Esa ambigüedad genera
expectativas falsas.

## What Changes

- **Logo compacto por debajo de 480px**: la barra muestra el logo sin el texto
  «OxiMeal» (se conserva el nombre accesible `aria-label="OxiMeal"`).
- **Grupo de acciones no compresible**: `.app-header__actions { flex-shrink: 0 }`
  para que Configuración y Salir nunca se compriman ni se desplacen.
- **Espaciado reducido en pantallas estrechas**: gaps y padding menores por
  debajo de 480px, manteniendo objetivos táctiles de 48px.
- **Sin cambio de comportamiento**: `Salir` sigue renderizándose sólo con
  `status === 'authenticated'`; no hay logout en ajustes ni en modo abierto.
- Tests: escenario de viewport estrecho (logo compacto, Configuración y Salir
  presentes y adyacentes) y escenario sin sesión (no hay `Salir`).

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `ui-shell`: el requirement «Shell responsive» pasa a exigir que Configuración y
  Salir queden siempre dentro del viewport, con logo compacto por debajo de
  480px y grupo de acciones no compresible.
- `auth-ui`: el requirement «Identidad y cierre de sesión en la interfaz» se
  explicita: sin sesión autenticada (modo `disabled`, `anonymous` o `loading`)
  no se ofrece el control `Salir`.

## Impact

- Frontend: `frontend/src/components/AppHeader.tsx` (logo compacto),
  `frontend/src/components/AppHeader.css` (breakpoint 480px, `flex-shrink`),
  `frontend/src/components/AppHeader.test.tsx` (nuevos escenarios).
- Specs: `openspec/specs/ui-shell/spec.md` y `openspec/specs/auth-ui/spec.md`
  (MODIFIED de los requirements citados).
- Sin cambios de backend, API, dependencias ni migraciones.
