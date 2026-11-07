# PLAN-004 — Botón «Marcar todo» en la compra

## Objetivo
Añadir a la vista de compra una acción «Marcar todo» junto a «Limpiar comprados», que marque como comprados todos los ítems pendientes, también sin conexión.

## Alcance
- Solo frontend (`ShoppingPage.tsx` + tests). Sin API ni backend nuevos.
- Reutilizar el toggle por ítem (`useToggleShoppingItem`) en serie.

## Tareas (TDD + SDD)
- [x] Change OpenSpec `shopping-mark-all` (delta MODIFIED `shopping-mode`) validado.
- [x] RED: tests (lote, deshabilitado, offline, rollback parcial, guard).
- [x] GREEN: botón + `handleMarkAll` secuencial + guard.
- [x] REFACTOR: `tsc`, `eslint`, `prettier`, `vitest` (404 OK).
- [x] Auditoría `@react-reviewer` (APROBADO tras 2 rondas).
- [x] PR #113 (feature) y #114 (archivado SDD) mergeados en `development`.
- [x] Doc `docs/shopping-mark-all.md`.
- [ ] Despliegue: reconstruir imagen `:local` y recrear contenedor.
