# OxiMeal — recetas de desarrollo.
# Ejecuta `just` (sin argumentos) para ver la lista de recetas disponibles.

# Muestra la lista de recetas.
default:
    @just --list

# Compila el workspace.
build:
    cargo build

# Arranca el backend.
run:
    cargo run -p oximeal-backend

# Arranca el backend con logging detallado.
dev:
    RUST_LOG=debug cargo run -p oximeal-backend

# Ejecuta la suite de tests del backend.
test:
    cargo test -p oximeal-backend

# Verifica la compilación de todos los targets sin ejecutar tests.
check:
    cargo check --all-targets

# Linter estricto (falla ante cualquier warning).
lint:
    cargo clippy --all-targets -- -D warnings

# Formatea el código.
fmt:
    cargo fmt

# Verifica el formato sin modificar ficheros.
fmt-check:
    cargo fmt --check

# Puerta de calidad completa: formato + lint + tests.
ci: fmt-check lint test

# Aplica las migraciones con la CLI de `sqlx` si está disponible.
migrate:
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-sqlite://data/oximeal.db}"
    if command -v sqlx >/dev/null 2>&1; then
        db_path="${DATABASE_URL#sqlite://}"
        db_path="${db_path%%\?*}"
        if [ -n "$db_path" ] && [ "$db_path" != ":memory:" ]; then
            mkdir -p "$(dirname "$db_path")"
        fi
        # sqlx CLI no crea el fichero salvo con `mode=rwc`.
        if [[ "$DATABASE_URL" == *"?"* ]]; then
            url="$DATABASE_URL"
        else
            url="${DATABASE_URL}?mode=rwc"
        fi
        sqlx migrate run --source backend/migrations --database-url "$url"
    else
        echo "sqlx CLI no disponible: las migraciones se aplican al arrancar el backend." >&2
    fi
