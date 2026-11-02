# Change: Linting de accesibilidad y formato con Prettier

## Why
El frontend no tiene linting estático de accesibilidad (`eslint-plugin-jsx-a11y`) ni formato automático (Prettier). La auditoría `react-reviewer` (Fase 2, M30) detectó `aria-*` mal usados que una regla de a11y habría señalado, y la tarea 1.4b (Prettier) quedó aplazada. El formato depende del editor y no hay verificación en CI (issue #11).

## What Changes
- Añadir `eslint-plugin-jsx-a11y`, `prettier` y `eslint-config-prettier` como devDependencies.
- Activar `jsxA11y.flatConfigs.recommended` en el flat config de ESLint y desactivar las reglas de formato en conflicto con `eslint-config-prettier`.
- Añadir `.prettierrc.json` y `.prettierignore`, y los scripts `format` / `format:check`.
- Formatear el árbol y corregir los avisos de a11y que aparezcan.
- Integrar `pnpm format:check` en `just ci-ui` y en el workflow de CI.

## Impact
- Capability: `frontend-tooling` (nueva).
- Ficheros: `frontend/package.json`, `frontend/pnpm-lock.yaml`, `frontend/eslint.config.js`, `frontend/.prettierrc.json`, `frontend/.prettierignore`, `justfile`, `.github/workflows/ci.yml`, y los ficheros de `frontend/src` que Prettier reformatee o que corrijan reglas de a11y.
- Sin cambios de comportamiento de producto.
