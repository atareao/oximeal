# Tasks — a11y + Prettier (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [x] 1.1 Línea base: `pnpm typecheck`, `pnpm test`, `pnpm lint` y `pnpm build` en verde
- [x] 1.2 Revisar el flat config actual y la versión de ESLint (10.x)

## 2. Dependencias y configuración
- [x] 2.1 Añadir `eslint-plugin-jsx-a11y`, `prettier` y `eslint-config-prettier`
- [x] 2.2 Resolver el peer de ESLint 10 para `eslint-plugin-jsx-a11y` (pnpm `peerDependencyRules`)
- [x] 2.3 Activar `jsxA11y.flatConfigs.recommended` y `eslint-config-prettier` en `eslint.config.js`
- [x] 2.4 Crear `.prettierrc.json` y `.prettierignore`
- [x] 2.5 Añadir scripts `format` y `format:check`

## 3. RED → GREEN
- [x] 3.1 Ejecutar `pnpm lint` y registrar los avisos de a11y (RED)
- [x] 3.2 Corregir los avisos de a11y en componentes y tests
- [x] 3.3 Ejecutar `pnpm format` y revisar el diff
- [x] 3.4 `pnpm lint` y `pnpm format:check` en verde (GREEN)

## 4. Integración y cierre
- [x] 4.1 Añadir `pnpm format:check` a `just ci-ui`
- [x] 4.2 Añadir el paso de Prettier al workflow de CI
- [x] 4.3 `just ci-ui` exit 0
- [x] 4.4 `openspec validate a11y-lint-prettier --strict` y `openspec validate --all --strict`
- [x] 4.5 Actualizar `PLAN.md` y `CHANGELOG.md`; archivar el change
