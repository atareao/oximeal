-- OxiMeal · Búsqueda de texto completo (FTS5, external content)
-- Tokenizer `trigram` para coincidencia difusa por substring (>= 3 caracteres).
-- El índice se mantiene sincronizado con `recipes` mediante triggers.

CREATE VIRTUAL TABLE recipes_fts USING fts5(
    name,
    instructions,
    content='recipes',
    content_rowid='rowid',
    tokenize='trigram'
);

-- AFTER INSERT: indexa la fila recién creada.
CREATE TRIGGER recipes_ai AFTER INSERT ON recipes BEGIN
    INSERT INTO recipes_fts (rowid, name, instructions)
    VALUES (new.rowid, new.name, new.instructions);
END;

-- AFTER DELETE: elimina la fila del índice (fila especial 'delete').
CREATE TRIGGER recipes_ad AFTER DELETE ON recipes BEGIN
    INSERT INTO recipes_fts (recipes_fts, rowid, name, instructions)
    VALUES ('delete', old.rowid, old.name, old.instructions);
END;

-- AFTER UPDATE: reindexa eliminando la versión antigua e insertando la nueva.
CREATE TRIGGER recipes_au AFTER UPDATE ON recipes BEGIN
    INSERT INTO recipes_fts (recipes_fts, rowid, name, instructions)
    VALUES ('delete', old.rowid, old.name, old.instructions);
    INSERT INTO recipes_fts (rowid, name, instructions)
    VALUES (new.rowid, new.name, new.instructions);
END;
