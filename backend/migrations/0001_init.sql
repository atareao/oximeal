-- OxiMeal · Migración inicial (esquema de la spec §3).
-- Tablas base: recetas, entradas del plan semanal y lista de la compra.

CREATE TABLE recipes (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    category           TEXT NOT NULL CHECK (category IN ('main', 'salad', 'side', 'dessert', 'breakfast')),
    diet_type          TEXT NOT NULL DEFAULT 'mediterranean',
    prep_time_minutes  INTEGER,
    servings           INTEGER DEFAULT 2,
    ingredients        JSON NOT NULL,
    instructions       TEXT,
    source_url         TEXT,
    created_at         DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE meal_plan_entries (
    id          TEXT PRIMARY KEY,
    date        DATE NOT NULL,
    meal_type   TEXT NOT NULL CHECK (meal_type IN ('lunch', 'dinner', 'breakfast', 'snack')),
    recipe_id   TEXT NOT NULL REFERENCES recipes(id),
    role        TEXT NOT NULL CHECK (role IN ('main', 'salad', 'side')),
    servings    INTEGER DEFAULT 2,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE shopping_list_items (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    amount          REAL NOT NULL,
    unit            TEXT NOT NULL,
    category        TEXT NOT NULL DEFAULT 'Otros',
    checked         BOOLEAN NOT NULL DEFAULT 0,
    source_plan_id  TEXT,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_meal_plan_date ON meal_plan_entries(date);
CREATE INDEX idx_recipes_category ON recipes(category);
CREATE INDEX idx_shopping_category ON shopping_list_items(category, checked);
