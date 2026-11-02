# OxiMeal

Gestor **self-hosted** de menús semanales, recetas y listas de la compra:
mobile-first, offline-capable y pensado para desplegarse como **binario único**
(Axum + SQLite).

## Arquitectura

OxiMeal es **headless / API-first**: expone una API HTTP tipada y documentada
(OpenAPI + Swagger UI) y no incluye orquestador LLM, servidor MCP ni chat.
El asistente conversacional es **Valet** (proyecto hermano), que consume la
**Agent API** de OxiMeal (`/api/agent`) mediante su patrón nativo
`services/<svc>.rs` + `tools/<svc>.rs`.

> Estado actual: **Fase 1 — núcleo backend**. Incluye el scaffold del workspace,
> el dominio puro (canonicalización de unidades, normalización de nombres,
> categorización por pasillo y consolidación de la lista de la compra), la API
> REST y la Agent API para Valet.

## Estructura

```
oximeal/
├── Cargo.toml            # workspace
├── rust-toolchain.toml    # toolchain stable
├── justfile               # recetas de desarrollo
├── backend/               # crate `oximeal-backend` (lib + bin)
│   ├── migrations/        # migraciones SQL versionadas
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── domain/        # dominio puro, sin IO
│       ├── models/        # DTOs (serde + utoipa)
│       ├── repo/          # acceso a datos (sqlx)
│       └── routes/        # handlers HTTP (axum)
└── frontend/              # reservado para la Fase 2 (React)
```

## Requisitos

- Rust **stable** (edition 2024). El sufijo `rust-toolchain.toml` fija el canal.
- [`just`](https://github.com/casey/just) para las recetas de desarrollo.
- `sqlx-cli` (opcional) para aplicar migraciones manualmente.

## Desarrollo

Copia el fichero de entorno y ajusta los valores:

```bash
cp .env.example .env
```

Recetas disponibles con [`just`](https://github.com/casey/just):

```bash
just            # lista las recetas disponibles
just build      # compila el workspace
just run        # arranca el backend
just dev        # arranca el backend con RUST_LOG=debug
just test       # suite de tests del backend
just check      # cargo check --all-targets
just lint       # cargo clippy --all-targets -- -D warnings
just fmt        # formatea el código
just fmt-check  # verifica el formato
just ci         # fmt-check + lint + test
just migrate    # aplica migraciones con sqlx (si está instalada)
```

Sin `just`, los comandos equivalentes son:

```bash
cargo build
cargo test -p oximeal-backend
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Configuración

| Variable | Descripción |
|---|---|
| `DATABASE_URL` | Ruta del fichero SQLite (p. ej. `sqlite://data/oximeal.db`) |
| `BIND_ADDR` | Dirección de escucha HTTP (p. ej. `0.0.0.0:8080`) |
| `RUST_LOG` | Filtro de logs de `tracing` (p. ej. `info,oximeal_backend=debug`) |

Las migraciones se aplican automáticamente al arrancar el backend.

## Licencia

MIT.
