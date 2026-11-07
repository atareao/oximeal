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

- **Reutilizar el toggle por ítem, en serie**: «Marcar todo» recorre los ítems pendientes y aplica el toggle **secuencialmente** (`await mutateAsync` uno tras otro) reutilizando `useToggleShoppingItem`, de modo que solo haya una mutación viva a la vez. Alternativa descartada: `POST /api/shopping-list/check-all`, operación compuesta no encolable e inservible sin conexión. Alternativa descartada: N `mutate()` en paralelo desde el mismo hook (coexisten mutaciones, observadores y `contextRef` compartido → riesgo de decidir mal el encolado).
- **Guard durante el lote**: mientras `isMarkingAll`, `handleToggle`/`handleCategoryChange` se ignoran, para no solapar una mutación del usuario con el lote.
- **Habilitación**: el botón se deshabilita cuando no hay ítems pendientes (sin ítems que marcar), igual que «Limpiar comprados» se deshabilita sin comprados.
- **Sin confirmación**: marcar no es destructivo (reversible desmarcando), así que no lleva `Popconfirm`.

## Risks / Trade-offs

- **N peticiones online** (una por ítem pendiente, serializadas por el `scope` del toggle): aceptable para el tamaño típico de una lista; a cambio se mantiene el soporte offline y cero backend nuevo.
- **Idempotencia**: solo se marcan los pendientes; los ya marcados no se tocan.

## Follow-up (fuera de alcance)

`useToggleShoppingItem` comparte la decisión `deferred` entre `onMutate` y `mutationFn` mediante un `contextRef` único. Con un lote secuencial no se dispara, pero dos toggles individuales concurrentes (usuario rápido) aún podrían pisar ese contexto. No afecta al rollback (`onError`/`onSuccess` reciben el contexto por argumento). Se documenta para una futura limpieza del hook (derivar `deferred` de las variables).
