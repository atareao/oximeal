# Tasks

## 1. Frontend: botón «Marcar todo» (TDD)
- [x] 1.1 RED: tests de `ShoppingPage` con ítems pendientes → al pulsar «Marcar todo» se marcan todos los pendientes (toggle por ítem), los ya marcados no cambian; deshabilitado sin pendientes; sin conexión se encola.
- [x] 1.2 GREEN: añadir el botón junto a «Limpiar comprados» y el handler que marca todos los pendientes reutilizando `useToggleShoppingItem`.
- [x] 1.3 REFACTOR: `npx tsc --noEmit`, `npx eslint .`, `npx prettier --check`, `npx vitest run`.

## 2. Verificación y despliegue
- [x] 2.1 `npx @fission-ai/openspec@1.16.0 validate --all --strict`.
- [x] 2.2 Auditoría `@react-reviewer`.
- [ ] 2.3 Reconstruir la imagen `ghcr.io/atareao/oximeal:local` y recrear el contenedor `oximeal`.
