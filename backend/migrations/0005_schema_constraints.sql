-- no-transaction
-- OxiMeal · Migración 0005: restricciones de integridad del esquema (issue #4).
--
-- Añade ON DELETE RESTRICT a meal_plan_entries.recipe_id y CHECK de dominio en
-- amount/servings/prep_time_minutes. SQLite no admite ALTER TABLE ADD CONSTRAINT,
-- así que se reconstruyen las tablas con el procedimiento de 12 pasos.
--
-- `-- no-transaction` desactiva la transacción automática de sqlx para poder
-- ejecutar `PRAGMA foreign_keys = OFF;` FUERA de transacción (SQLite ignora el
-- PRAGMA dentro de una). Todo el DDL va envuelto manualmente en BEGIN/COMMIT
-- para que sea atómico: si una fila existente viola los nuevos CHECK, el
-- `INSERT ... SELECT` falla y provoca ROLLBACK, sin dejar la BD a medias.
-- `PRAGMA foreign_key_check` verifica las FK antes de confirmar.
-- Append-only: no edita migraciones previas.

PRAGMA foreign_keys = OFF;

BEGIN;

-- 1) recipes (se retira el índice FTS5 external-content para reconstruirlo después).
DROP TRIGGER IF EXISTS recipes_ai;
DROP TRIGGER IF EXISTS recipes_ad;
DROP TRIGGER IF EXISTS recipes_au;
DROP TABLE IF EXISTS recipes_fts;

CREATE TABLE recipes_new (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    category           TEXT NOT NULL CHECK (category IN ('main', 'salad', 'side', 'dessert', 'breakfast')),
    diet_type          TEXT NOT NULL DEFAULT 'mediterranean',
    prep_time_minutes  INTEGER CHECK (prep_time_minutes IS NULL OR prep_time_minutes >= 0),
    servings           INTEGER DEFAULT 2 CHECK (servings IS NULL OR servings >= 1),
    ingredients        JSON NOT NULL,
    instructions       TEXT,
    source_url         TEXT,
    created_at         DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO recipes_new (id, name, category, diet_type, prep_time_minutes, servings, ingredients, instructions, source_url, created_at)
    SELECT id, name, category, diet_type, prep_time_minutes, servings, ingredients, instructions, source_url, created_at FROM recipes;

DROP TABLE recipes;
ALTER TABLE recipes_new RENAME TO recipes;

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

-- 2) meal_plan_entries (ON DELETE RESTRICT + CHECK servings).
CREATE TABLE meal_plan_entries_new (
    id          TEXT PRIMARY KEY,
    date        DATE NOT NULL,
    meal_type   TEXT NOT NULL CHECK (meal_type IN ('lunch', 'dinner', 'breakfast', 'snack')),
    recipe_id   TEXT NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
    role        TEXT NOT NULL CHECK (role IN ('main', 'salad', 'side')),
    servings    INTEGER DEFAULT 2 CHECK (servings IS NULL OR servings >= 1),
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO meal_plan_entries_new (id, date, meal_type, recipe_id, role, servings, created_at)
    SELECT id, date, meal_type, recipe_id, role, servings, created_at FROM meal_plan_entries;

DROP TABLE meal_plan_entries;
ALTER TABLE meal_plan_entries_new RENAME TO meal_plan_entries;

-- 3) shopping_list_items (CHECK amount >= 0).
CREATE TABLE shopping_list_items_new (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    amount          REAL NOT NULL CHECK (amount >= 0),
    unit            TEXT NOT NULL,
    category        TEXT NOT NULL DEFAULT 'Otros',
    checked         BOOLEAN NOT NULL DEFAULT 0,
    source_plan_id  TEXT,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO shopping_list_items_new (id, name, amount, unit, category, checked, source_plan_id, updated_at)
    SELECT id, name, amount, unit, category, checked, source_plan_id, updated_at FROM shopping_list_items;

DROP TABLE shopping_list_items;
ALTER TABLE shopping_list_items_new RENAME TO shopping_list_items;

-- Índices (se pierden al DROP TABLE; se recrean todos).
CREATE INDEX IF NOT EXISTS idx_meal_plan_date ON meal_plan_entries(date);
CREATE INDEX IF NOT EXISTS idx_recipes_category ON recipes(category);
CREATE INDEX IF NOT EXISTS idx_recipes_created_at ON recipes(created_at DESC, id ASC);
CREATE INDEX IF NOT EXISTS idx_shopping_category ON shopping_list_items(category, checked);
CREATE INDEX IF NOT EXISTS idx_shopping_source_plan_id ON shopping_list_items(source_plan_id);
CREATE INDEX IF NOT EXISTS idx_shopping_checked ON shopping_list_items(checked);

-- Verifica que ninguna fila viole las FK antes de confirmar.
PRAGMA foreign_key_check;
COMMIT;

PRAGMA foreign_keys = ON;
