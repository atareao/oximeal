# Change: Documentar la Vibration API como decisión de haptics

## Why
La spec `shopping-mode` dice "Web Haptics", pero la implementación usa la Vibration API (`navigator.vibrate`) en `useHaptics.ts`. Funcionalmente equivalente en Android y no-op en iOS Safari. El paquete `web-haptics` está en 0.0.6 (API inmadura) y añadiría una dependencia y un workaround frágil para iOS. Se decide documentar la Vibration API y alinear spec y código (issue #10).

## What Changes
- Actualizar el requisito "Marcar como comprado" de `shopping-mode` para hablar de Vibration API (`navigator.vibrate`) y de no-op cuando no hay soporte.
- Actualizar el docstring de `useHaptics.ts` con la decisión y el comportamiento en iOS Safari.
- Añadir tests del patrón por defecto y de patrones en array.

## Impact
- Capability: `shopping-mode` (requisito modificado).
- Ficheros: `openspec/specs/shopping-mode/spec.md`, `frontend/src/hooks/useHaptics.ts`, `frontend/src/hooks/useHaptics.test.ts`, `PLAN.md`, `CHANGELOG.md`.
- Sin dependencias nuevas ni cambios de comportamiento.
