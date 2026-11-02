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

Las variables de autenticación (Fase 4) se documentan en la sección
[🔐 Autenticación (OIDC con Pocket ID)](#-autenticación-oidc-con-pocket-id).

## 🔐 Autenticación (OIDC con Pocket ID)

OxiMeal usa el backend como **BFF (Backend For Frontend)**: el flujo
*authorization code + PKCE* y el intercambio de tokens ocurren en el servidor,
la sesión se guarda **server-side en SQLite** y el navegador solo recibe una
cookie `httpOnly` + `Secure` + `SameSite=Lax`. **El navegador nunca ve tokens.**

### Activar la autenticación

| `OIDC_ENABLED` | Comportamiento |
|---|---|
| `false` (def.) | API **abierta**, sin autenticación. **Solo para desarrollo local.** |
| `true` | BFF activo: cookie de sesión (SPA) o `Authorization: Bearer` (M2M/Valet). |

Con `OIDC_ENABLED=true` son **obligatorias** `OIDC_ISSUER`, `OIDC_CLIENT_ID`,
`OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`,
`OIDC_POST_LOGOUT_REDIRECT_URI`, `OIDC_ALLOWED_GROUPS` y `SESSION_SECRET`.
El servidor **falla al arrancar** si falta alguna o si `SESSION_SECRET` es un
valor inseguro/por defecto. Arranca igualmente aunque el IdP esté caído (la
sesión caducará, pero el proceso no muere).

Variables opcionales con sus valores por defecto: `SESSION_TTL_SECS`
(`2592000` = 30 días), `SESSION_REFRESH_MARGIN_SECS` (`300` = 5 min),
`SESSION_COOKIE_NAME` (`oximeal_session`), `SESSION_COOKIE_SECURE` (`true`) y
`ALLOWED_ORIGINS` (CSV; vacío ⇒ ningún origen CORS permitido).

### Permisos por endpoint

La autorización es **exclusivamente por permisos**; no hay roles ni
administración de instancia. Sin credencial válida → **401** con
`WWW-Authenticate: Bearer`; credencial válida sin permiso → **403**.

| Método y ruta | Permiso |
|---|---|
| `GET /api/recipes`, `GET /api/recipes/{id}` | `read:recipes` |
| `POST /api/recipes` | `write:recipes` |
| `GET /api/meal-plan` | `read:plan` |
| `POST /api/meal-plan`, `DELETE /api/meal-plan/{id}` | `write:plan` |
| `GET /api/shopping-list` | `read:shopping` |
| `POST /api/shopping-list`, `PATCH /api/shopping-list/{id}`, `POST /api/shopping-list/consolidate`, `DELETE /api/shopping-list/completed` | `write:shopping` |
| `GET /api/agent/recent-recipes`, `GET /api/agent/search-recipes` | `agent:read` |
| `POST /api/agent/weekly-plan` | `agent:plan` |
| `POST /api/agent/shopping-items`, `DELETE /api/agent/shopping/completed` | `agent:shopping` |
| `GET /api/health`, `GET /api/openapi.json`, `/api/docs*`, `/auth/*` | **público** |

Los permisos se leen del claim `scope` (modelo *APIs and permissions* de
Pocket ID). Los permisos `agent:*` están pensados para clientes **M2M** y no se
conceden a clientes user-delegated. Las mutaciones autenticadas por cookie
exigen la cabecera `X-CSRF-Token` (el SPA la obtiene de `GET /auth/me`).

**Allowlist de grupos:** `OIDC_ALLOWED_GROUPS` (CSV, p. ej.
`oximeal-users,oximeal-admin`) es la **puerta de entrada**: solo los usuarios
que pertenecen a alguno de estos grupos pueden autenticarse. No concede
permisos por sí misma ni implementa roles; un usuario fuera de la allowlist es
rechazado aunque su token sea válido.

### Migración del API key estático (issue #5)

El API key estático `OXIMEAL_API_KEY` queda **obsoleto** y **ya no se acepta**:
no existe ninguna vía de autenticación por cabecera de API key. Los clientes
que lo usaban deben migrar a OIDC:

- **SPA / navegador:** flujo *authorization code + PKCE* contra el BFF
  (`GET /auth/login` → `GET /auth/callback` → cookie de sesión).
- **Cliente máquina (Valet y similares):** **M2M `client_credentials`** con
  permisos `agent:*` y `Authorization: Bearer`.

Ejemplo de obtención de token M2M (`client_credentials`). El endpoint de token
es el que anuncia el documento de descubrimiento del IdP:

```bash
# 1) Descubre el token_endpoint del IdP.
TOKEN_ENDPOINT=$(curl -s "$OIDC_ISSUER/.well-known/openid-configuration" \
  | python3 -c 'import sys, json; print(json.load(sys.stdin)["token_endpoint"])')

# 2) Solicita el token con las credenciales del cliente M2M.
curl -s -X POST "$TOKEN_ENDPOINT" \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'grant_type=client_credentials' \
  -d "client_id=$OIDC_CLIENT_ID" \
  -d "client_secret=$OIDC_CLIENT_SECRET" \
  -d 'scope=agent:read agent:plan agent:shopping'
```

Ejemplo de llamada a la Agent API con el token M2M:

```bash
curl -s -X POST "http://localhost:8080/api/agent/weekly-plan" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"preferences":{"servings":2}}'
```

### IdP de desarrollo

Para levantar un Pocket ID de desarrollo (junto con la configuración de cliente,
API/resource, grupos y cliente M2M) consulta [`deploy/oidc-dev.md`](deploy/oidc-dev.md).
El `compose` de desarrollo está en [`deploy/compose.yaml`](deploy/compose.yaml):

```bash
podman compose -f deploy/compose.yaml up -d   # o: docker compose -f deploy/compose.yaml up -d
```

Los valores de ejemplo de `.env.example` (`OIDC_ISSUER=http://localhost:1411`,
`OIDC_REDIRECT_URI=http://localhost:8080/auth/callback`, etc.) apuntan a ese
IdP de desarrollo.

### Nota sobre HTTPS

Pocket ID **exige HTTPS en los redirect URIs** fuera de `localhost`: en
despliegues reales hay que servirse a través de un **reverse proxy TLS** (p. ej.
Caddy o Traefik) y registrar los redirect URIs con `https://`. `localhost` queda
**exento** para el desarrollo local, por lo que ejemplos como
`http://localhost:8080/auth/callback` son válidos solo en local.

### Dependencia de la Fase 5 (PWA/offline)

Los **access tokens caducan a los 60 minutos** y el **refresh token dura 30 días**
de inactividad. La Fase 5 (PWA/offline) deberá **tolerar la sesión caducada**
sirviendo datos **cacheados** mientras no sea posible reautenticar, en lugar de
fallar o vaciar la interfaz.

## Licencia

MIT.
