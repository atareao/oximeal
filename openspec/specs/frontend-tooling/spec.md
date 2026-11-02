# frontend-tooling Specification

## Purpose

Esta capacidad define el tooling de calidad del frontend: linting estático de accesibilidad con `eslint-plugin-jsx-a11y` sobre los componentes TSX y verificación de formato con Prettier, ambas integradas en `just ci-ui` y en el workflow de CI.

## Requirements

### Requirement: Linting de accesibilidad
El frontend SHALL ejecutar ESLint con las reglas recomendadas de `eslint-plugin-jsx-a11y` sobre todos los ficheros TSX (`**/*.tsx`), y `pnpm lint` SHALL terminar con 0 problemas.

#### Scenario: Regla de a11y activa
- GIVEN un componente con un atributo ARIA inválido
- WHEN se ejecuta `pnpm lint`
- THEN ESLint reporta el problema como error

#### Scenario: Árbol limpio
- GIVEN el árbol de `frontend/src` sin violaciones
- WHEN se ejecuta `pnpm lint`
- THEN el comando termina con 0 problemas

### Requirement: Formato con Prettier
El proyecto SHALL definir una configuración de Prettier y SHALL verificar el formato con `pnpm format:check`, integrado en `just ci-ui` y en el workflow de CI.

#### Scenario: Formato correcto
- GIVEN el árbol formateado con Prettier
- WHEN se ejecuta `pnpm format:check`
- THEN el comando termina con exit 0

#### Scenario: Formato incorrecto
- GIVEN un fichero con formato distinto al configurado
- WHEN se ejecuta `pnpm format:check`
- THEN el comando falla y señala el fichero
