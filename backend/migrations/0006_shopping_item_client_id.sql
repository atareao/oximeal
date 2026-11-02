-- OxiMeal · Migración 0006: identificador de cliente para el alta idempotente.
--
-- Permite que un alta reenviada tras un fallo de red (cola de escrituras
-- offline) no duplique ítems. NULL para los ítems creados sin client_id, y el
-- índice único es parcial para no colisionar entre NULLs.
--
-- Append-only: no edita migraciones previas.

ALTER TABLE shopping_list_items ADD COLUMN client_id TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_shopping_items_client_id
    ON shopping_list_items (client_id)
    WHERE client_id IS NOT NULL;
