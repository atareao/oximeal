# Tasks — Imagen de contenedor y despliegue (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Leer `Cargo.toml`, `backend/Cargo.toml`, `rust-toolchain.toml`, `justfile` (`ui-build`, `build-full`) y `.env.example` para fijar contratos (feature `embed-ui`, `DATABASE_URL`, `BIND_ADDR`, OIDC, `SESSION_*`)
- [ ] 1.2 Leer `frontend/package.json`, `frontend/pnpm-lock.yaml` y `frontend/scripts/build-sw.mjs` (Node/pnpm y salida `frontend/dist`)
- [ ] 1.3 Leer `backend/src/routes/health.rs` (`GET /api/health`) y `backend/src/static_files.rs` (fallback SPA) para los escenarios de healthcheck y servido de UI
- [ ] 1.4 Leer `deploy/compose.yaml`, `deploy/oidc-dev.md` y `deploy/.env.example` para decidir servicio `oximeal`, red y volúmenes
- [ ] 1.5 Verificar disponibilidad de Docker/Podman con BuildKit en el entorno de verificación
- [ ] 1.6 Confirmar `ghcr.io/atareao/oximeal` como nombre de imagen y registrar ADR-32

## 2. RED
- [ ] 2.1 Test/verificación: `docker build -t ghcr.io/atareao/oximeal:test .` falla hoy por ausencia de `Dockerfile` (documentar el rojo)
- [ ] 2.2 Test: el stage `frontend` produce `frontend/dist` (build de la UI con `--frozen-lockfile`)
- [ ] 2.3 Test: el stage `builder` produce el binario release con `--features embed-ui`
- [ ] 2.4 Test: el contenedor arranca como usuario no root (UID 10001) y sin toolchain Rust/Node
- [ ] 2.5 Test: `GET /api/health` en el contenedor responde 200 y el `HEALTHCHECK` lo marca `healthy`
- [ ] 2.6 Test: `DATABASE_URL=sqlite://data/oximeal.db` persiste datos en el volumen `/app/data` entre reinicios
- [ ] 2.7 Test: el contenedor sirve `/` y una ruta SPA (`/recipes`), y `/api/*` no cae al fallback
- [ ] 2.8 Test: `.dockerignore` excluye `target/`, `node_modules/`, `frontend/dist/`, `data/`, `.env` y `deploy/.env`
- [ ] 2.9 Test: `docker compose -f deploy/compose.yaml config` valida el servicio `oximeal` junto a `pocket-id`
- [ ] 2.10 Ejecutar la batería de verificación y confirmar el estado RED

## 3. GREEN
- [ ] 3.1 Crear `Dockerfile` multi-stage (`frontend` → `builder` → `runtime`) con bases fijadas (node, rust stable, `debian:bookworm-slim`)
- [ ] 3.2 `runtime`: instalar solo `ca-certificates` y `curl`, crear usuario `oximeal` (10001), `WORKDIR /app`, `EXPOSE 8080`, `VOLUME /app/data`, `ENV` por defecto
- [ ] 3.3 Añadir `HEALTHCHECK` contra `/api/health` y `ENTRYPOINT ["/usr/local/bin/oximeal-backend"]`
- [ ] 3.4 Orden de capas para caché: manifiestos y lockfiles antes que el código fuente
- [ ] 3.5 Crear `.dockerignore` con artefactos, metadatos, datos y secretos
- [ ] 3.6 Añadir el servicio `oximeal` a `deploy/compose.yaml` (build/imagen, `env_file`, volumen `oximeal-data:/app/data`, red `devnet`, `depends_on`/healthcheck) conservando `pocket-id`
- [ ] 3.7 Actualizar `deploy/.env.example` con las variables de la aplicación (modo abierto y OIDC)
- [ ] 3.8 Build de la imagen en verde; `bash`/inspección confirman usuario no root, binario único y healthcheck en `healthy`

## 4. REFACTOR
- [ ] 4.1 Verificar el tamaño/tageo de la imagen y reducir capas superfluas si procede
- [ ] 4.2 Documentar en `README.md` (sección de despliegue + refresco de estado a Fase 6/7) y crear `deploy/README.md`
- [ ] 4.3 Re-verificar build, healthcheck y persistencia tras el refactor (sin regresiones)
- [ ] 4.4 Revisar que el modo abierto y el modo OIDC arrancan según lo documentado

## 5. Cierre
- [ ] 5.1 `openspec validate container-image --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` (Fase 7, ADR-32) y `CHANGELOG.md`
- [ ] 5.3 Anotar en el change que la publicación en GHCR corresponde a `release-automation`
- [ ] 5.4 Archivar el change con `openspec archive container-image`
