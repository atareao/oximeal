
# Imagen multi-stage de OxiMeal (openspec/changes/container-image, ADR-32).
# Contexto de build = raíz del monorepo.

### Stage 1 - frontend (SPA con Vite) ###
FROM node:24-alpine AS frontend
WORKDIR /app/frontend
RUN npm install -g pnpm@12
# Manifiestos + lockfile antes que el codigo: install reproducible y cacheable.
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

### Stage 2 - builder (backend Rust con la UI embebida) ###
FROM rust:bookworm AS builder
WORKDIR /app
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl \
 && rm -rf /var/lib/apt/lists/*
# Manifiestos primero (cache de dependencias).
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/build.rs ./backend/
# Compila un stub para cachear las dependencias de Cargo; su resultado se descarta.
RUN mkdir -p backend/src \
 && printf 'fn main() {}\n' > backend/src/main.rs \
 && printf '\n' > backend/src/lib.rs \
 && cargo build --release -p oximeal-backend --features embed-ui || true
# Codigo real + migraciones (sqlx::migrate!) + UI ya construida por el stage frontend.
COPY backend/src ./backend/src
COPY backend/migrations ./backend/migrations
COPY --from=frontend /app/frontend/dist ./frontend/dist
RUN cargo build --release -p oximeal-backend --features embed-ui

### Stage 3 - runtime (minimo, no root) ###
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --gid 10001 oximeal \
 && useradd --uid 10001 --gid oximeal --create-home --shell /usr/sbin/nologin oximeal
WORKDIR /app
RUN mkdir -p /app/data && chown oximeal:oximeal /app/data
COPY --from=builder /app/target/release/oximeal-backend /usr/local/bin/oximeal-backend
ENV DATABASE_URL="sqlite://data/oximeal.db" \
    BIND_ADDR="0.0.0.0:8080" \
    RUST_LOG="info"
EXPOSE 8080
VOLUME ["/app/data"]
USER 10001:10001
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -f http://127.0.0.1:8080/api/health || exit 1
ENTRYPOINT ["/usr/local/bin/oximeal-backend"]

