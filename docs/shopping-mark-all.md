# Botón «Marcar todo» en la compra

## Qué es
Acción de la vista de la compra que marca como **comprados** todos los ítems **pendientes** de una sola vez, junto a «Limpiar comprados».

## Ficheros implicados
- `frontend/src/pages/ShoppingPage.tsx` — `pendingCount`, estado `isMarkingAll`, `handleMarkAll` y el botón.
- `frontend/src/pages/ShoppingPage.test.tsx` — tests de lote, deshabilitado, offline, rollback parcial y guard.
- `frontend/src/api/hooks.ts` — se reutiliza `useToggleShoppingItem` (sin cambios).

## Cómo funciona
- `handleMarkAll` recorre los ítems pendientes (`checked === false`) y aplica el toggle **secuencialmente** (`await toggleItem.mutateAsync({ id, patch: { checked: true } })`), de modo que solo hay una mutación viva a la vez.
- Sin conexión (o con id local) el toggle **encola** el cambio en IndexedDB y lo aplica de forma optimista (`offline-writes`); al volver la conexión se sincroniza.
- El botón se **deshabilita** cuando no hay pendientes y entra en `loading` (estado local `isMarkingAll`) mientras dura el lote.
- Mientras `isMarkingAll`, `handleToggle`/`handleCategoryChange` se ignoran (guard) para no solapar una mutación del usuario con el lote.
- Sin confirmación: marcar no es destructivo (se puede desmarcar).

## Variables / configuración
Ninguna. Sin cambios de API, backend ni esquema.

## Cómo habilitar / desplegar
Frontend embebido en el binario: reconstruir la imagen y recrear el contenedor.
```
podman build --format docker -t ghcr.io/atareao/oximeal:local .
podman stop oximeal && podman rm oximeal
podman run -d --name oximeal -p 8080:8080 -v oximeal-data:/app/data --restart unless-stopped -e OIDC_ENABLED=false ghcr.io/atareao/oximeal:local
```

## Decisiones
- **Reutilizar el toggle por ítem en serie** (no endpoint bulk `check-all`): mantiene el soporte offline y cero backend nuevo. Descartado N `mutate()` en paralelo desde el mismo hook (coexisten mutaciones/observadores y `contextRef` compartido en `useToggleShoppingItem`).
- Guard durante el lote para evitar solapes.

## Follow-ups
- `useToggleShoppingItem` comparte la decisión `deferred` entre `onMutate` y `mutationFn` vía un `contextRef` único; dos toggles individuales concurrentes podrían pisarlo (no afecta al rollback). Limpieza futura: derivar `deferred` de las variables.
