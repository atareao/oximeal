# Proposal

## Why

En el modo compra, marcar todos los productos de una vez exige tocar uno a uno. Junto a «Limpiar comprados» falta una acción que marque como comprado todo lo pendiente, para cerrar la compra con un solo gesto.

## What Changes

- Añadir a la barra de acciones de la compra un botón **«Marcar todo»** junto a «Limpiar comprados» que marca como comprados todos los ítems pendientes.
- Reutiliza el toggle por ítem existente (y su cola offline), por lo que funciona sin conexión. Sin cambios de API ni de backend.

## Capabilities

### New Capabilities
<!-- Ninguna. -->

### Modified Capabilities
- `shopping-mode`: la acción «Alta rápida y limpieza» incorpora «Marcar todo».

## Impact

- Frontend: `frontend/src/pages/ShoppingPage.tsx` (+ tests) y, si aplica, un helper en `frontend/src/api/hooks.ts` o `frontend/src/lib/shopping.ts`. Reutiliza `useToggleShoppingItem`.
- Sin cambios de backend, API ni esquema.
