-- OxiMeal · Migración 0004: almacén de sesiones server-side (tower-sessions).
--
-- Append-only: no edita migraciones previas ni toca las tablas de dominio.
-- `data` guarda el registro de sesión serializado por `tower-sessions`;
-- `expiry` es el instante Unix (segundos) de caducidad.

CREATE TABLE sessions (
    id      TEXT PRIMARY KEY,
    data    BLOB NOT NULL,
    expiry  INTEGER NOT NULL
);

CREATE INDEX idx_sessions_expiry ON sessions(expiry);
