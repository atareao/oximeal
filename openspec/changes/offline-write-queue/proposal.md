# Change: Cola de escrituras offline

## Why
Con la lectura offline resuelta, falta poder escribir sin conexión: marcar ítems, añadir productos y limpiar comprados. Hoy cualquier mutación sin red falla y se pierde. Se necesita una cola persistente que se sincronice al recuperar la conexión sin duplicar ítems.

## What Changes
- Backend: `POST /api/shopping-list` acepta un `client_id` opcional y único; si el ítem ya existe, devuelve el existente sin crear un duplicado (idempotencia).
- Backend: migración que añade `client_id` a `shopping_list_items` con índice único parcial.
- Frontend: cola FIFO de mutaciones en IndexedDB, con aplicación optimista en la caché local y drenado al recuperar la conexión.
- Frontend: reconciliación del identificador provisional con el real tras el alta y last-write-wins por `updated_at`.
- Frontend: indicador de cambios pendientes de sincronizar en la vista de la compra.

## Impact
- Capabilities: nueva `offline-writes`; delta en `shopping-list`.
- Backend: `backend/migrations/0005_shopping_item_client_id.sql`, `backend/src/models`, `backend/src/repo/shopping_list.rs`, `backend/src/routes/shopping_list.rs`.
- Frontend: `frontend/src/offline/mutationQueue.ts`, `frontend/src/api/hooks.ts`, `frontend/src/pages/ShoppingPage.tsx`.
- `client_id` es opcional y retrocompatible: el alta sin él sigue creando un ítem nuevo.
