-- OxiMeal · Migración 0010: ajustes del hogar (household_settings).
--
-- Tabla singleton (id = 1) para preferencias globales del hogar.
-- Append-only: no modifica migraciones previas.

CREATE TABLE IF NOT EXISTS household_settings (
    id                 INTEGER PRIMARY KEY CHECK (id = 1),
    default_servings   INTEGER NOT NULL DEFAULT 2 CHECK (default_servings BETWEEN 1 AND 20),
    updated_at         TEXT NOT NULL
);

-- Siembra idempotente de la fila singleton con valor por defecto.
INSERT OR IGNORE INTO household_settings (id, default_servings, updated_at)
VALUES (1, 2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
