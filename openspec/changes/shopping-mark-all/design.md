# Design

## Context

`ShoppingPage` ofrece «Consolidar desde el menú» y «Limpiar comprados». El marcado por ítem usa `useToggleShoppingItem`, que aplica actualización optimista y, sin conexión o con id local, **encola** el cambio en IndexedDB (`offline-writes` reserva la cola para mutaciones por ítem: alta, toggle y edición de cantidad).

## Goals / Non-Goals

**Goals:**
- Marcar como comprado todo lo pendiente con un solo gesto, también sin conexión.

**Non-Goals:**
- Endpoint bulk de servidor (`check-all`): evitar nueva superficie de API y churn en `offline-writes`.
- Desmarcar todo (solo se marca).

## Decisions

- **Reutilizar el toggle por ítem**: «Marcar todo» recorre los ítems pendientes y aplica el toggle (o una actualización optimista única + toggles por ítem) reutilizando `useToggleShoppingItem`. Alternativa descartada: `POST /api/shopping-list/check-all`, que sería una operación compuesta no encolable y quedaría inservible sin conexión.
- **Habilitación**: el botón se deshabilita cuando no hay ítems pendientes (sin ítems que marcar), igual que «Limpiar comprados» se deshabilita sin comprados.
- **Sin confirmación**: marcar no es destructivo (reversible desmarcando), así que no lleva `Popconfirm`.

## Risks / Trade-offs

- **N peticiones online** (una por ítem pendiente, serializadas por el `scope` del toggle): aceptable para el tamaño típico de una lista; a cambio se mantiene el soporte offline y cero backend nuevo.
- **Idempotencia**: solo se marcan los pendientes; los ya marcados no se tocan.
