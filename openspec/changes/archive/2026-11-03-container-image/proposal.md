# Change: imagen de contenedor y despliegue de OxiMeal

## Why

OxiMeal se distribuye hoy solo como **binario compilado a mano**: `just
build-full` (`just ui-build` + `cargo build --release -p oximeal-backend
--features embed-ui`) produce un ejecutable con la UI embebida, pero **no hay
`Dockerfile` ni `.dockerignore`**. Cualquier despliegue real obliga al operador a
reproducir el toolchain (Rust stable + edition 2024, Node + pnpm) y a resolver a
mano los directorios de datos, el usuario de ejecución y el arranque. Esto
contradice el objetivo de OxiMeal como self-hosted de **binario único** y hace el
paso de entrega frágil y no reproducible.

Además, `deploy/compose.yaml` **solo despliega Pocket ID** (el IdP de desarrollo);
no hay forma declarativa de levantar la aplicación junto a su IdP. El `README.md`
también está desactualizado («Estado actual: Fase 1», «frontend/ reservado para la
Fase 2»), lo que da una imagen equivocada del estado real del proyecto.

Este change define y documenta el **empaquetado de la aplicación en una imagen de
contenedor** reproducible (multi-stage, runtime mínimo, no root, healthcheck y
persistencia en volumen), su **orquestación con compose** junto a Pocket ID, y el
refresco de la documentación de despliegue. Es el change **E1 de la Fase 7
(entrega)** y materializa la decisión **ADR-32** sobre imagen y despliegue. Cierra
el issue #52.

## What Changes

- **`Dockerfile` multi-stage** en la raíz del repo (contexto = raíz del monorepo):
  1. **`frontend`**: `node` + `pnpm`, `pnpm install --frozen-lockfile` y
     `pnpm build` → `frontend/dist`.
  2. **`builder`**: imagen oficial `rust` (línea stable acorde a
     `rust-toolchain.toml`); copia `Cargo.toml`/`Cargo.lock`/`backend/`, recibe
     `frontend/dist` desde el stage `frontend` y ejecuta
     `cargo build --release -p oximeal-backend --features embed-ui`.
  3. **`runtime`**: `debian:bookworm-slim` con **solo** `ca-certificates` y
     `curl`; copia únicamente el binario final; usuario **no root**, `WORKDIR
     /app`, `EXPOSE 8080`, `VOLUME /app/data`, `ENV` por defecto
     (`DATABASE_URL=sqlite://data/oximeal.db`, `BIND_ADDR=0.0.0.0:8080`),
     `HEALTHCHECK` contra `/api/health` y `ENTRYPOINT`.
- **`.dockerignore`** en la raíz que excluye artefactos de build (`target/`,
  `frontend/node_modules/`, `frontend/dist/`, `frontend/tsconfig.tsbuildinfo`),
  metadatos (`.git/`, `.github/`, `openspec/`), datos y secretos (`data/`,
  `.env`, `deploy/.env`), y documentación no necesaria para el build.
- **Caché de capas**: copiar primero los manifiestos (`Cargo.toml`, `Cargo.lock`,
  `frontend/package.json`, `frontend/pnpm-lock.yaml`) e instalar/compilar antes de
  copiar el código fuente.
- **`deploy/compose.yaml`**: se añade un servicio `oximeal` (build local o imagen
  de GHCR, `env_file`, volumen con nombre para SQLite, red `devnet` y
  `depends_on`/healthcheck) **manteniendo** el servicio `pocket-id`; se documenta
  el arranque conjunto y el arranque en modo abierto (sin OIDC).
- **Documentación**: nueva sección de despliegue en `README.md` (con refresco del
  estado a Fase 6/7) y `deploy/README.md`; se actualiza `deploy/.env.example` con
  las variables de la aplicación.

## Impact

- **Capabilities**: nueva capacidad `deployment` (`specs/deployment/spec.md`).
- **Ficheros nuevos**: `Dockerfile`, `.dockerignore`, `deploy/README.md`.
- **Ficheros modificados**: `deploy/compose.yaml`, `deploy/.env.example`,
  `README.md`, `PLAN.md` y `CHANGELOG.md` (cierre).
- **Tooling**: requiere Docker/Podman con BuildKit (multi-stage). No introduce
  dependencias de código ni cambios en el binario, la API o la UI.
- **Dependencias**: ninguna nueva en Cargo ni en el frontend. El runtime usa
  `debian:bookworm-slim` + `ca-certificates` + `curl`; el builder, imágenes
  oficiales `node`/`rust`.
- **Relación con otros changes**: la **publicación** de la imagen en GHCR la
  asume el change `release-automation` (Fase 7); este change define el build
  reproducible y su consumo local/declarativo.
- **Decisión**: ADR-32 (imagen y despliegue).
- Cierra el issue #52.
