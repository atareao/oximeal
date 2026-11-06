# Proposal

## Why

La paleta oscura actual tiene un marcado tinte azulado (`#0f172a`, `#111827`, `#1e293b`), heredado de la escala *slate* de Tailwind. En pantallas OLED y con el brillo bajo habitual, ese azul produce una dominante fría que resulta artificial y reduce la sensación de contraste percibido. Se quiere una escala neutra de negros y grises, coherente entre las variables CSS, los tokens de Ant Design, el `theme-color` y el manifest PWA, manteniendo el acento verde y sin sacrificar la accesibilidad AA.

## What Changes

- Sustituir la paleta oscura azulada por una **escala neutra** de negros y grises con contraste **AA** (ratio ≥ 4.5:1 para texto normal):
  - `--bg`: `#0f172a` → `#0a0a0a`
  - `--surface`: `#111827` → `#141414`
  - `--surface-muted`: `#1e293b` → `#1f1f1f`
  - `--text`: `#e2e8f0` → `#e5e5e5`
  - `--text-muted`: `#94a3b8` → `#a3a3a3`
  - `--border`: `#334155` → `#3f3f3f`
  - `--accent`: `#48bb78` (se mantiene, verde)
  - `--accent-soft`: `#1c3d2e` → `#14251c`
- Aplicar los mismos hex neutros a `darkTheme` en `frontend/src/components/theme.ts` (`colorBgBase`, `colorTextBase` y `Layout.bodyBg/headerBg/siderBg`).
- Actualizar `THEME_COLORS.dark` en `frontend/src/theme/ThemeProvider.tsx` (alimenta `<meta name="theme-color">`).
- Mantener la coherencia PWA: si cambia el `theme-color`, actualizar a la vez `frontend/index.html` (`<meta name="theme-color">`) y `frontend/public/manifest.webmanifest` (`theme_color`), que el test `frontend/src/offline/manifest.test.ts` exige idénticos.
- Sin cambios de comportamiento: el selector de tema (Oscuro/Claro/Sistema), la persistencia en `localStorage` y `color-scheme` siguen intactos.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `ui-shell`: el requisito **Tema Antd y safe-area** se amplía para exigir que, en modo oscuro, los fondos sean neutros (sin tinte azul), que el texto alcance contraste AA y que `theme-color`/manifest sigan coherentes.

## Impact

- Frontend (estilos y tema): `frontend/src/index.css` (bloque `:root[data-theme='dark']`), `frontend/src/components/theme.ts` (`darkTheme`), `frontend/src/theme/ThemeProvider.tsx` (`THEME_COLORS.dark`).
- Frontend (PWA): `frontend/index.html` (meta `theme-color`) y `frontend/public/manifest.webmanifest` (`theme_color`), que deben coincidir entre sí.
- Tests: se añade cobertura que veta tintes azulados y/o valida el contraste AA en modo oscuro. No hay tests que fijen hex concretos salvo la coincidencia manifest ↔ `index.html`.
- Sin cambios de backend, API, base de datos ni dependencias.
