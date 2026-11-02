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

# Ejecuta la suite de tests del backend con la UI embebida.
test-embed-ui:
    cargo test -p oximeal-backend --features embed-ui

# Verifica la compilación de todos los targets sin ejecutar tests.
check:
    cargo check --all-targets --all-features

# Linter estricto (falla ante cualquier warning).
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Formatea el código.
fmt:
    cargo fmt

# Verifica el formato sin modificar ficheros.
fmt-check:
    cargo fmt --check

# Puerta de calidad completa: formato + lint + tests (ambas features).
ci: fmt-check lint test test-embed-ui

# Instala las dependencias del frontend.
ui-install:
    cd frontend && pnpm install

# Compila el frontend a `frontend/dist`.
ui-build:
    cd frontend && pnpm build

# Compila el binario con la UI embebida (requiere `ui-build`).
build-full: ui-build
    cargo build --release -p oximeal-backend --features embed-ui

# Mide el RSS (estable y pico) del binario release con la UI embebida.
# NO es un gate de CI (ADR-31): verificación manual de rendimiento.
perf-mem: build-full
    ./scripts/perf-rss.sh

# Audita FCP, transferencia inicial y objetivos táctiles por CDP (Chrome
# headless; el propio script arranca y para el binario). NO es un gate de CI
# (ADR-31).
perf-web: build-full
    node scripts/perf-web.mjs

# Verificación completa de rendimiento (RSS + web). NO es un gate de CI
# (ADR-31): se ejecuta a mano sobre el binario release.
perf: perf-mem perf-web

# Puerta de calidad del frontend: tipos + formato + tests + lint + build.
ci-ui:
    cd frontend && pnpm typecheck && pnpm format:check && pnpm test && pnpm lint && pnpm build && pnpm check-bundle

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
