-- OxiMeal · Plegado de diacríticos en FTS5 + índices de rendimiento.
--
-- La migración 0002 creó el índice con `tokenize='trigram'`, que no pliega
-- acentos: `cafe` no encontraba `café`. Se recrea el índice externo con
-- `remove_diacritics 1` (SQLite >= 3.45) y se reindexa el contenido existente.
-- Es append-only: no se edita 0002.

DROP TRIGGER IF EXISTS recipes_ai;
DROP TRIGGER IF EXISTS recipes_ad;
DROP TRIGGER IF EXISTS recipes_au;
DROP TABLE IF EXISTS recipes_fts;

CREATE VIRTUAL TABLE recipes_fts USING fts5(
    name, instructions, content='recipes', content_rowid='rowid',
    tokenize='trigram remove_diacritics 1'
);

CREATE TRIGGER recipes_ai AFTER INSERT ON recipes BEGIN
    INSERT INTO recipes_fts(rowid, name, instructions) VALUES (new.rowid, new.name, new.instructions);
END;

CREATE TRIGGER recipes_ad AFTER DELETE ON recipes BEGIN
    INSERT INTO recipes_fts(recipes_fts, rowid, name, instructions) VALUES('delete', old.rowid, old.name, old.instructions);
END;

CREATE TRIGGER recipes_au AFTER UPDATE ON recipes BEGIN
    INSERT INTO recipes_fts(recipes_fts, rowid, name, instructions) VALUES('delete', old.rowid, old.name, old.instructions);
    INSERT INTO recipes_fts(rowid, name, instructions) VALUES (new.rowid, new.name, new.instructions);
END;

INSERT INTO recipes_fts(recipes_fts) VALUES('rebuild');

-- Índices que cubren el listado de recetas por fecha y los filtros de la compra.
CREATE INDEX IF NOT EXISTS idx_recipes_created_at ON recipes(created_at DESC, id ASC);
CREATE INDEX IF NOT EXISTS idx_shopping_source_plan_id ON shopping_list_items(source_plan_id);
CREATE INDEX IF NOT EXISTS idx_shopping_checked ON shopping_list_items(checked);
