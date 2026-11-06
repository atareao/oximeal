# Tareas — dark-theme-neutral

## 1. RED (tests que fallan)

- [x] 1.1 Nuevo `frontend/src/theme/darkPalette.test.ts`: parsear el bloque `:root[data-theme='dark']` de `index.css` y `theme.ts`, y vetar fondos con dominante azul (p. ej. `B > R` de forma perceptible); verificar que falla con la paleta actual.
- [x] 1.2 En el mismo test, calcular el ratio de contraste WCAG de `--text`/`--text-muted` sobre `--bg`/`--surface`/`--surface-muted` y exigir ≥ 4.5:1; verificar que pasa con la paleta actual (regresión de contraste).
- [x] 1.3 Ejecutar `npx vitest run` y confirmar el rojo esperado solo por el veto de tinte azul.

## 2. GREEN (implementación mínima)

- [x] 2.1 Editar `frontend/src/index.css` bloque `:root[data-theme='dark']` con la escala neutra (`#0a0a0a`, `#141414`, `#1f1f1f`, `#e5e5e5`, `#a3a3a3`, `#3f3f3f`, `#14251c`); verificar con `npx vitest run` que 1.1 pasa.
- [x] 2.2 Editar `darkTheme` en `frontend/src/components/theme.ts` (`colorBgBase`, `colorTextBase`, `Layout.bodyBg/headerBg/siderBg`) con los mismos hex.
- [x] 2.3 Editar `THEME_COLORS.dark` en `frontend/src/theme/ThemeProvider.tsx` a `#0a0a0a`; verificar que el `<meta name="theme-color">` se actualiza con el tema resuelto.
- [x] 2.4 Editar a la vez `theme_color` en `frontend/public/manifest.webmanifest` y `<meta name="theme-color">` en `frontend/index.html` al mismo valor neutro; verificar con `npx vitest run frontend/src/offline/manifest.test.ts` que la coincidencia se cumple.
- [x] 2.5 `npx tsc --noEmit` sin errores.

## 3. REFACTOR

- [x] 3.1 Revisar que no quedan hex azulados residuales en el modo oscuro (`rg '#0f172a|#111827|#1e293b|#334155' frontend/src frontend/index.html frontend/public`); corregir los que pertenezcan a la paleta oscura.
- [x] 3.2 `npm run lint` sin warnings nuevos y `npm run format:check` en verde.
- [x] 3.3 `npx vitest run` completo en verde.

## 4. Cierre (integración)

- [ ] 4.1 Verificar de forma observable en el navegador que el modo oscuro no tiene tinte azul y que el cambio de tema (Oscuro/Claro/Sistema) sigue funcionando. _(verificación manual, fuera del alcance automatizable de esta sesión)_
- [x] 4.2 `openspec validate dark-theme-neutral --strict` sin errores.
- [x] 4.3 Actualizar `CHANGELOG.md` en `[No publicado]`.
