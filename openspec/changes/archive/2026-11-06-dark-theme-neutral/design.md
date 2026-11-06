# Design

## Context

Ver `proposal.md` — Why. El grueso del chrome de UI se pinta con variables CSS propias (bloque `:root[data-theme='dark']` en `index.css`) y con tokens de Ant Design (`darkTheme` en `components/theme.ts`). El color del sistema operativo se sincroniza vía `color-scheme` y `<meta name="theme-color">`, alimentado por `THEME_COLORS.dark` en `theme/ThemeProvider.tsx`. La PWA duplica ese color en `public/manifest.webmanifest`, y un test exige que manifest y meta coincidan exactamente.

## Goals / Non-Goals

**Goals:**

- Una única escala neutra, reutilizada en los tres puntos de edición (CSS, tokens Antd, theme-color) para evitar derivas.
- Contraste AA (≥ 4.5:1) en texto normal sobre cualquier superficie oscura.
- Cero cambios de comportamiento en el selector de tema y en `color-scheme`.

**Non-Goals:**

- Rediseñar la paleta clara.
- Introducir un sistema de tokens nuevo o migrar de Ant Design.
- Cambiar tipografía, radios, alturas táctiles o layout.

## Decisions

### Escala neutra propuesta

Se adopta una escala de grises puros (canales R=G=B salvo el acento), alineada con la escala *neutral* de Tailwind:

| Variable / token | Antes (azulado) | Después (neutro) | Uso |
| --- | --- | --- | --- |
| `--bg` / `colorBgBase` / `Layout.bodyBg` | `#0f172a` | `#0a0a0a` | Fondo base |
| `--surface` / `Layout.siderBg` | `#111827` | `#141414` | Superficie elevada |
| `--surface-muted` | `#1e293b` | `#1f1f1f` | Superficie atenuada |
| `--text` / `colorTextBase` | `#e2e8f0` | `#e5e5e5` | Texto principal |
| `--text-muted` | `#94a3b8` | `#a3a3a3` | Texto secundario |
| `--border` | `#334155` | `#3f3f3f` | Bordes y separadores |
| `--accent` | `#48bb78` | `#48bb78` | Acento (verde, sin cambio) |
| `--accent-soft` | `#1c3d2e` | `#14251c` | Fondo de acento atenuado |

`Layout.headerBg` pasa de `#111827` a `#141414` (misma superficie que el resto del chrome).

### Contraste AA (verificado)

Ratios calculados con la fórmula WCAG 2.1 (luminancia relativa sRGB):

| Combinación | Ratio | AA (normal, ≥ 4.5) |
| --- | --- | --- |
| `--text` `#e5e5e5` sobre `--bg` `#0a0a0a` | 15.72:1 | ✅ |
| `--text` `#e5e5e5` sobre `--surface` `#141414` | 14.62:1 | ✅ |
| `--text` `#e5e5e5` sobre `--surface-muted` `#1f1f1f` | 13.08:1 | ✅ |
| `--text-muted` `#a3a3a3` sobre `--bg` `#0a0a0a` | 7.85:1 | ✅ |
| `--text-muted` `#a3a3a3` sobre `--surface` `#141414` | 7.30:1 | ✅ |
| `--text` `#e5e5e5` sobre `--accent-soft` `#14251c` | 12.71:1 | ✅ |
| `--accent` `#48bb78` sobre `--bg` `#0a0a0a` | 8.16:1 | ✅ |

Todas las combinaciones superan AA con holgura; `--text-muted` también cumple AAA (≥ 7:1) sobre el fondo base.

### Dónde se edita (exhaustivo)

1. `frontend/src/index.css` — bloque `:root[data-theme='dark']`.
2. `frontend/src/components/theme.ts` — `darkTheme`: `colorBgBase`, `colorTextBase`, `Layout.bodyBg/headerBg/siderBg`.
3. `frontend/src/theme/ThemeProvider.tsx` — `THEME_COLORS.dark`.
4. `frontend/index.html` — `<meta name="theme-color" content="...">` (hoy `#2f855a`).
5. `frontend/public/manifest.webmanifest` — `theme_color` (hoy `#2f855a`).

### Decisión: theme-color tras el cambio

Hoy el `theme-color` y el `manifest.theme_color` son `#2f855a` (verde institucional) y no dependen del modo. El test `offline/manifest.test.ts` solo exige que **manifest y meta coincidan**, no un valor concreto. La UI oscura es el modo por defecto, así que alinear el chrome con la paleta neutra (`#0a0a0a`) evita una costura verde entre la barra del sistema y el contenido. **Alternativa considerada:** dejar `#2f855a`. Se descarta porque rompe la continuidad visual del modo por defecto.

`THEME_COLORS.dark` debe pasar a `#0a0a0a` para que el `<meta>` dinámico (via `ThemeProvider`) siga coincidiendo con el manifest estático. `THEME_COLORS.light` no cambia.

## Risks / Trade-offs

- **[Deriva entre los 5 ficheros]** → Una sola escala en `design.md` como fuente de verdad; el test de contraste/neutralidad detecta regresiones.
- **[El test de manifest ↔ index.html]** → Cambiar `#2f855a` exige editar ambos ficheros a la vez; se hace en la misma tarea.
- **[Hex repetido en CSS y TS]** → Ya es el patrón actual del proyecto (no hay generación de tokens); se asume la duplicación controlada.

## Open Questions

- Ninguna que afecte a specs, enfoque o desglose de tareas.
