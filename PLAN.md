# PLAN.md — OxiMeal

> Plan maestro de implementación. Generado por el Project Director.
> Spec de origen: «Especificación Técnica y Funcional: OxiMeal v1.0».
> **Alcance ejecutable en esta ronda: Fase 7 — entrega (Docker/release) y producto (recetario/menú).**

## 1. Visión y alcance

OxiMeal es un gestor self-hosted de menús semanales, recetas y listas de la compra, mobile-first, offline-capable y desplegable como **binario único** (Axum + `rust-embed`), con un orquestador LLM interno (OpenRouter) conectado a un **servidor MCP** que ejecuta acciones reales sobre SQLite.

**En esta ronda:** cerrar la entrega (imagen de contenedor + automatización de
release) y completar el bucle del recetario/menú en la app. Las fases anteriores
(F1–F6) están cerradas; ver §19 y §20.

## 2. Decisiones de arquitectura (ADR resumen)

| # | Decisión | Justificación |
|---|----------|---------------|
| ADR-1 | Cargo **workspace** con `members = ["backend"]` y carpeta `frontend/` reservada | Permite añadir crates (shared, xtask) sin reestructurar; separa backend (F1) de UI (F2). |
| ADR-2 | **sqlx** (runtime-tokio, sqlite, macros, migrate) con migraciones versionadas | Encaja con Axum/Tokio; queries parametrizadas en runtime (sin macros query! ni .sqlx). |
| ADR-3 | FTS5 **external content** + tokenizer `trigram` + triggers ai/ad/au | Búsqueda difusa por substring en nombre/instrucciones; el índice se mantiene por triggers. |
| ADR-4 | **Motor de consolidación determinista** en `src/domain/` | Normaliza unidades y suma cantidades; 100% testeable con TDD. |
| ADR-5 | **OxiMeal headless**: sin orquestador/LLM propio; el asistente es **Valet** vía integración nativa HTTP | Evita duplicar el asistente de Valet, que descartó MCP y usa integración nativa por servicio (services+tools). |
| ADR-6 | **OpenAPI v3 (utoipa)** + Swagger UI en `/api/docs` desde Fase 1 | La spec lo lista bajo Axum; barato ahora y facilita el consumo desde MCP. |
| ADR-7 | SQLite con `journal_mode=WAL`, `foreign_keys=on`, `busy_timeout` | Robustez en self-hosted con lecturas concurrentes. |
| ADR-8 | `rust-embed` diferido a Fase 2 | Fase 1 expone API pura; el empaquetado monolítico necesita el frontend. |
| ADR-9 | **Agent API** en `/api/agent` (5 endpoints) como contrato para Valet | Superficie estable y tipada para el orquestador; MCP sería un adapter opcional futuro sobre la misma API. |
| ADR-10 | Frontend **React 19 + Antd v6** con Vite/TS y datos vía TanStack Query; empaquetado en el binario con `rust-embed` (feature `embed-ui`) | Mobile-first y binario único; sin SSR (no se necesita). |
| ADR-11 | Autenticación **OIDC con Pocket ID** mediante **BFF**: cliente confidencial, sesión server-side y cookie `httpOnly`; autorización por **permisos del modelo *APIs and permissions***; Valet por **M2M `client_credentials`** | Instancia self-hosted hoy abierta; el SPA se sirve del mismo origen, así que el BFF no añade coste; cero tokens en el navegador. |

## 3. Estructura del repositorio

```
oximeal/
├── Cargo.toml                 # [workspace] members = ["backend"]
├── rust-toolchain.toml        # stable (>=1.97), edition 2024
├── justfile                   # dev / test / lint / fmt / migrate / check
├── .env.example               # DATABASE_URL, BIND_ADDR, RUST_LOG, OPENROUTER_*
├── .gitignore                 # /target, /data, node_modules, dist, .env
├── README.md
├── PLAN.md
├── GIT_FLOW.md
├── openspec/
│   ├── project.md
│   ├── specs/                 # capacidades consolidadas (tras archivar)
│   └── changes/phase-1-core-backend/{proposal.md,tasks.md,specs/…}
├── backend/
│   ├── Cargo.toml
│   ├── migrations/
│   │   ├── 0001_init.sql
│   │   └── 0002_fts_triggers.sql
│   └── src/
│       ├── main.rs            # bootstrap: config, tracing, pool, migrate, serve
│       ├── lib.rs             # factoría del Router (reusable en tests)
│       ├── config.rs
│       ├── db.rs
│       ├── error.rs
│       ├── state.rs
│       ├── domain/{mod,units,normalize,category,ingredient,consolidate}.rs
│       ├── models/{mod,recipe,meal_plan,shopping_list}.rs
│       └── routes/{mod,health,recipes,meal_plan,shopping_list}.rs
└── frontend/                  # placeholder (Fase 2)
    └── .gitkeep
```

## 4. Stack y dependencias (backend)

- **Runtime web:** `axum = "0.8"`, `tokio` (rt-multi-thread, macros, signal), `tower`, `tower-http` (trace, cors).
- **Datos:** `sqlx` (runtime-tokio, sqlite, macros, migrate, uuid, chrono, json).
- **Tipos/utilidades:** `serde`, `serde_json`, `uuid` (v4, serde), `chrono` (serde), `thiserror`, `anyhow`.
- **Observabilidad:** `tracing`, `tracing-subscriber` (env-filter).
- **Docs API:** `utoipa`, `utoipa-swagger-ui`.
- **Dev:** `axum-test` o `tower::ServiceExt::oneshot`, `tokio` (test-util), `tempfile`.
- **Fase 2 (aún no):** `rust-embed`, `mime_guess`.

## 5. Fase 1 — Desglose de tareas (ejecutable ahora)

- **T1 · Estructura base**
  - `Cargo.toml` raíz como workspace; mover el crate actual a `backend/`.
  - `rust-toolchain.toml`, `.gitignore`, `README.md`, `.env.example`.
- **T2 · Dependencias backend**
  - `backend/Cargo.toml` con el stack del punto 4.
- **T3 · Migraciones**
  - `0001_init.sql`: tablas `recipes`, `meal_plan_entries`, `shopping_list_items` con CHECKs e índices de la spec (§3).
  - `0002_fts_triggers.sql`: `CREATE VIRTUAL TABLE recipes_fts USING fts5(name, instructions, content='recipes', content_rowid='rowid', tokenize='trigram')` + triggers `recipes_ai/recipes_ad/recipes_au`.
  - Verificar en runtime disponibilidad de FTS5 y del tokenizer `trigram` (SQLite >= 3.34).
- **T4 · Config y pool**
  - `config.rs` (env: `DATABASE_URL`, `BIND_ADDR`, `RUST_LOG`, `OPENROUTER_API_KEY`, `OPENROUTER_MODEL`).
  - `db.rs`: `SqliteConnectOptions` (`create_if_missing`, WAL, foreign_keys, busy_timeout) + `sqlx::migrate!()`.
- **T5 · Infra HTTP**
  - `AppError` (enum → `IntoResponse`; mapea `sqlx::Error`, 404, validación).
  - `AppState { pool, config }`; router factoría en `lib.rs`.
  - `tracing` + `tower_http::trace` (request-id) + CORS.
- **T6 · Dominio (TDD, sin IO)**
  - `units.rs`: `UnitKind{Mass,Volume,Count,Spoon,Teaspoon,Pinch,ToTaste,Unknown}`, factores canónicos (kg→g×1000, l→ml×1000, cl→ml×10, cda/cdta, etc.) y parseo de strings de unidad.
  - `normalize.rs`: lowercase, trim, colapso de espacios, eliminación de diacríticos, artículos/stopwords, singularización básica.
  - `category.rs`: tabla de reglas ordenada keyword→pasillo (`🥦 Verdulería`, `🐟 Pescadería`, `🥩 Carnicería`, `🥫 Despensa`, `❄️ Refrigerados`, `🍞 Panadería`, `🥤 Bebidas`, `Otros`).
  - `ingredient.rs`: `Ingredient`, `IngredientInput`.
  - `consolidate.rs`: `consolidate(&[IngredientInput]) -> Vec<ShoppingListItemDraft>` (agrupa por nombre normalizado, suma por familia de unidad, respeta no-aditivos, categoriza, ordena estable).
- **T7 · Modelos y repositorios**
  - DTOs serde + `ToSchema`; repos async por entidad.
- **T8 · Rutas Recetas** (§6)
  - `GET /api/recipes?q=&category=&page=&page_size=` (FTS5 `MATCH`; fallback `LIKE` si `q.len()<3`).
  - `POST /api/recipes` (JSON; preparado para import Schema.org).
  - `GET /api/recipes/:id`.
- **T9 · Rutas Planificación**
  - `GET /api/meal-plan?start_date=&end_date=`, `POST /api/meal-plan`, `DELETE /api/meal-plan/:id`.
- **T10 · Rutas Lista de la Compra**
  - `GET /api/shopping-list` (agrupada por categoría), `PATCH /api/shopping-list/:id` (toggle/cantidad),
    `POST /api/shopping-list/consolidate` (regenera desde el plan), `DELETE /api/shopping-list/completed`.
- **T11 · Salud y documentación**
  - `GET /api/health`; OpenAPI en `/api/openapi.json` + Swagger UI en `/api/docs`.
- **T12 · Tooling**
  - `justfile` con recetas `dev/test/lint/fmt/migrate/check`.
- **T13 · Agent API (cierre F1)** — `/api/agent`: `GET recent-recipes?days=`, `GET search-recipes?query=&category=`, `POST weekly-plan` (bulk transaccional + reconstruye la compra), `POST shopping-items` (inserta/suma), `DELETE shopping/completed`.

## 6. Fases 2–4 (alto nivel, planificadas)

- **Fase 1.5 — Agent API** (cerrada junto con la Fase 1 en esta ronda).
- **Fase 2 — Frontend (COMPLETADA):** React 19 + Antd v6 (Vite/TS) en `frontend/`; shell responsive (Sider en desktop / bottom-nav en móvil); menú semanal lunes–domingo; modo compra mobile-first por pasillos (objetivos ≥48px, strike-through + reordenado + haptics); servido con `rust-embed` (feature `embed-ui`) y fallback SPA; la pestaña Asistente enlaza/embebe Valet. La deuda diferida (ETag/304, `charset=utf-8`, Range/nosniff/CSP, `eslint-plugin-jsx-a11y`, Prettier, CI remoto) quedó registrada como issues.
- **Fase 3 — Integración con Valet (en el repo Valet, NO en OxiMeal):** cliente `src/services/oximeal.rs` + tools `src/tools/oximeal.rs` (con permisos) que consumen la Agent API. **No** se construye orquestador/MCP/chat en OxiMeal (Valet descartó MCP; usa integración nativa por servicio). La pestaña 🤖 del PWA enlaza/embebe el chat de Valet.
- **Fase 4 — Autenticación OIDC (Pocket ID):** BFF con cliente confidencial, sesión server-side y cookie `httpOnly` (el navegador nunca ve tokens); autorización por **permisos** del modelo *APIs and permissions* (user-delegated para el SPA y **M2M `client_credentials`** para Valet); puerta de entrada por grupos permitidos (*Allowed user groups* del cliente + `OIDC_ALLOWED_GROUPS`), **sin roles** ni administración de instancia; CSRF y guard de rutas en el frontend; `compose` con un IdP de desarrollo. Cierra el issue #5 (API key estático). Dependencia registrada: la Fase 5 deberá tolerar sesión caducada (access token 60 min, refresh 30 días) sirviendo datos cacheados.
- **Fase 5 — PWA/Offline (COMPLETADA):** `manifest.json` + iconos adaptativos; Service Worker (Stale-While-Revalidate para activos, Network-First con fallback IndexedDB para la lista); pruebas táctiles y de rendimiento. Debe tolerar la sesión OIDC caducada sirviendo datos cacheados mientras no haya autenticación.

## 7. Estrategia de testing

- **Unit (dominio):** `#[cfg(test)]` en `domain/*` — canonicalización de unidades, normalización de nombres, categorización, consolidación (sumas, no-aditivos, conflictos de unidad).
- **Integración (API):** `backend/tests/*.rs` levantan el `Router` de `lib.rs` con SQLite temporal migrada (`tempfile`) y usan `oneshot`/`axum-test`.
- **Contrato FTS:** caso que inserta recetas y verifica coincidencia por substring (trigram) y fallback LIKE.
- **Frontend:** `vitest` + Testing Library (`@testing-library/react` / `user-event`); se mockea `src/api/client` con `vi.mock` y se ejercitan hooks reales envueltos en `QueryClientProvider`.
- **Puertas obligatorias:** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `just ci`, `just ci-ui`, `openspec validate --all --strict`.

## 8. Git flow (resumen; detalle en GIT_FLOW.md)

- Ramas: `main` (producción) · `development` (integración) · `feature/*` desde `development` · `hotfix/*` desde `main`.
- Nunca commits directos a `main`/`development`: todo por PR.
- Commits: `<gitmoji> <tipo>(<scope>): <mensaje>` (Conventional Commits + gitmoji).

## 9. Criterios de aceptación (Definition of Done, Fase 1)

- [x] Workspace compila: `cargo build` OK.
- [x] Migraciones aplican sobre base nueva; FTS5+trigram verificados en runtime.
- [x] Motor de consolidación con cobertura de tests (unidades, acentos, no-aditivos, conflictos).
- [x] Los 12 endpoints REST de §6 + los 5 de la Agent API implementados y cubiertos por tests de integración.
- [x] `cargo test` en verde; `clippy -D warnings` limpio; `fmt --check` limpio.
- [x] `openspec validate phase-1-core-backend --strict` en verde.
- [x] `/api/health` responde 200; Swagger accesible en `/api/docs`.
- [x] Queries sqlx parametrizadas en runtime (sin dependencia de .sqlx).
- [x] Agent API `/api/agent` implementada y cubierta por tests de integración.

### Fase 2

- [x] `frontend/` compila sin errores: `pnpm typecheck` + `pnpm build` (vite).
- [x] `pnpm test` en verde (73 tests con Vitest + Testing Library).
- [x] `eslint` limpio.
- [x] Shell responsive verificado (Sider desktop / bottom-nav móvil).
- [x] Fallback SPA correcto (`/api/*` no cae al fallback).
- [x] `cargo test` sigue en verde con `embed-ui` on y off.
- [x] Code-splitting por rutas activo (chunks de `MenuPage` / `ShoppingPage`).
- [x] `just ci` y `just ci-ui` en verde.
- [x] Auditoría `react-reviewer` + `rust-reviewer` aplicada (0 bloqueantes).

### Fase 4

- [x] `openspec validate oidc-auth --strict` en verde.
- [x] 401 (sin credenciales), 403 (sin permiso) y 200 (con permiso) correctos por cada permiso (`read:recipes`, `write:recipes`, `read:plan`, `write:plan`, `read:shopping`, `write:shopping`).
- [x] Cookie de sesión `httpOnly` + `Secure` + `SameSite=Lax` verificada; el navegador nunca recibe tokens.
- [x] CSRF activo (`X-CSRF-Token`) en las mutaciones autenticadas por cookie.
- [x] M2M de la Agent API funcionando y contrato de peticiones/respuestas sin cambios.
- [x] `OIDC_ENABLED=false` mantiene `cargo test` en verde.
- [x] Frontend con guard de rutas y `401`→re-login.
- [x] IdP de desarrollo documentado en `compose`.

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| FTS5/trigram no disponible en el build de `libsqlite3-sys` | Task T3 verifica en runtime; fallback a `LIKE` para `q.len()<3`. |
| Query trigram insensible a acentos limitada | `normalize.rs` aplica folding de acentos antes de indexar/buscar. |
| Edición 2024 requiere toolchain reciente | `rust-toolchain.toml` fija stable ≥1.97 (presente en el entorno). |
| Ambigüedad Schema.org en import | Resuelto en la Fase 6 (change B): parser JSON-LD `Recipe` propio, con guardas SSRF, mapeo documentado e idempotencia por `source_url`. |
| Consumo de RAM <20 MB (objetivo global) | Medido sobre el binario monolítico (feature `embed-ui`, change `perf-device-verification`): RSS estable 11 MB / pico 11 MB con `[profile.release]` optimizado (baseline 12/12 MB), ~5 MB bajo el techo de 16 MB. Ver `docs/performance.md`. |
| Offline vs access token de 60 min | La Fase 5 debe tolerar la sesión caducada sirviendo datos cacheados; el refresh dura 30 días de inactividad. |
| Pocket ID exige HTTPS en los redirect URIs fuera de localhost | Reverse proxy HTTPS documentado para los despliegues; localhost queda exento para desarrollo. |
| Fuga de tokens por logs | Prohibir loguear las cabeceras `Cookie` y `Authorization` en tracing/middleware. |
| Bloqueo si el IdP cae | El servidor no depende del IdP al arrancar: arranca igualmente y reintenta descubrimiento/JWKS; las rutas protegidas responden de forma coherente. |

## 11. Trazabilidad con la especificación

- §3 Modelo de datos → T3, T7.
- §4 Tools MCP → Agent API HTTP (`/api/agent`), consumida por Valet.
- §5 UI mobile-first → Fase 2.
- §6 Endpoints REST → T8–T11.
- §7 Roadmap → secciones 5 y 6 de este plan.
- Autenticación OIDC (issue #5) → **Fase 4** (`openspec/changes/oidc-auth/`): BFF, permisos y M2M.

## 12. Fase 4 — Autenticación OIDC (Pocket ID): plan de ejecución por bloques

> Ejecutable ahora. Origen: `openspec/changes/oidc-auth/` (5 specs / 24 requirements / 44 escenarios).
> Metodología: SDD + TDD estricto (RED → GREEN → REFACTOR) bloque a bloque, con auditoría por bloque.

### 12.1 Objetivo

Cerrar la instancia self-hosted (hoy abierta: CORS `Any`, sin auth) con OIDC contra Pocket ID v2.19.1, usando el backend como **cliente confidencial BFF**: *authorization code + PKCE + state/nonce*, intercambio de código en servidor, **sesión server-side** y cookie `httpOnly`+`Secure`+`SameSite=Lax`. El navegador nunca ve tokens. Autorización **exclusivamente por permisos** (`read:recipes`, `write:recipes`, `read:plan`, `write:plan`, `read:shopping`, `write:shopping`; `agent:*` para M2M). Doble vía: cookie de sesión (SPA) o `Authorization: Bearer` (Valet). Sin roles ni administración de instancia.

### 12.2 Decisiones de diseño (ADR-12…17)

| # | Decisión | Justificación |
|---|----------|---------------|
| ADR-12 | Cliente OIDC **manual** (`reqwest` + `jsonwebtoken`), **sin** el crate `openidconnect` | Permite apuntar discovery/token/JWKS a un mock (wiremock) en tests de integración; control total del flujo. |
| ADR-13 | Sesión server-side con **`tower-sessions`** y un `SessionStore` propio sobre SQLite | Firma/cifrado de cookie idiomáticos, rotación con `cycle_id()`, persistencia y caducidad gestionadas; sin cripto propia. |
| ADR-14 | Autorización en **doble vía** (cookie o `Bearer`) con una **tabla única de permisos** | Misma política para SPA y Valet; los permisos se leen del claim `scope` (Pocket ID *APIs and permissions*). |
| ADR-15 | “Sesión pre-auth” para `state`/`nonce`/PKCE entre `/auth/login` y `/auth/callback` | Cumple “guardados en la sesión”; al autenticar se rota el identificador. |
| ADR-16 | CSRF `X-CSRF-Token` **solo** en mutaciones autenticadas por cookie (Bearer exento) | Bearer no es *ambient authority*: no sufre CSRF. |
| ADR-17 | Gate `OIDC_ENABLED`: `false` = API abierta + warning (compat tests); `true` = config obligatoria o fallo de arranque, y el arranque **no** depende del IdP | Cumple escenarios “OIDC desactivado”, “config incompleta” e “IdP caído al arrancar”. |

### 12.3 Módulos nuevos (backend)

```
backend/src/auth/
├── mod.rs        # AuthService (discovery+JWKS+token), constructor, estado
├── oidc.rs       # discovery document + JWKS con caché y refresco por kid
├── jwt.rs        # validación de access tokens RFC 9068 (iss/aud/exp/nbf/alg)
├── session.rs    # modelo de sesión + SessionStore (tower-sessions) sobre SQLite
├── csrf.rs       # comparación en tiempo constante
├── middleware.rs # AuthContext + layer require_permission(perm)
├── routes.rs     # /auth/login, /auth/callback, /auth/me, /auth/logout
└── error.rs      # AuthError → 401 (WWW-Authenticate: Bearer) / 403
backend/migrations/0004_sessions.sql
```

`AppState` gana `auth: Option<Arc<AuthService>>` (None ⇔ `OIDC_ENABLED=false`).

### 12.4 Configuración (env) y validación de arranque

Variables nuevas: `OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `ALLOWED_ORIGINS` (lista CSV), `OIDC_ALLOWED_GROUPS` (CSV), `SESSION_SECRET`, `SESSION_TTL_SECS` (def. 2592000 = 30 d), `SESSION_REFRESH_MARGIN_SECS` (def. 300), `SESSION_COOKIE_SECURE` (def. true), `SESSION_COOKIE_NAME` (def. `oximeal_session`).

`Config::from_env()` pasa a devolver `anyhow::Result<Config>`. Reglas:
- Con `OIDC_ENABLED=true`: obligatorios `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `OIDC_ALLOWED_GROUPS` y `SESSION_SECRET`; si falta alguno → `Err` con mensaje que liste los ausentes. `SESSION_SECRET` con valor vacío o por defecto conocido → `Err`.
- Con `OIDC_ENABLED=false`: no se exige secreto; se emite `tracing::warn!` de que la API queda abierta.

### 12.5 Migración `0004_sessions.sql` (no toca tablas de dominio)

```sql
CREATE TABLE sessions (
    id      TEXT PRIMARY KEY,
    data    BLOB NOT NULL,     -- registro de sesión serializado (tower-sessions)
    expiry  INTEGER NOT NULL   -- unix seconds
);
CREATE INDEX idx_sessions_expiry ON sessions(expiry);
```

Contenido lógico de la sesión: en pre-auth `{state, nonce, code_verifier, return_to, exp}`; autenticada `{subject, name, groups[], permissions[], csrf_token, access_token, refresh_token, id_token, access_token_expires_at}`. Los tokens **nunca** salen al navegador.

### 12.6 Tabla de permisos por endpoint

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
| `GET /api/health`, `/api/openapi.json`, `/api/docs*`, `/auth/*` | público |

Sin credencial válida → 401 + `WWW-Authenticate: Bearer`; credencial válida sin permiso → 403. Los permisos `agent:*` **no** se conceden a clientes user-delegated.

### 12.7 Bloques de ejecución (mapeo con `tasks.md`)

1. **Bloque 1 — Deps, config, migración y arranque** (tasks 1.1–1.4). Tests: config incompleta falla; secreto por defecto falla; `OIDC_ENABLED=false` → warning y API abierta.
2. **Bloque 2 — BFF y sesión** (tasks 2.1–2.7). Tests: login (state/nonce/PKCE S256), callback feliz con rotación, `state` inválido → 400, fuera de grupos → 403, `/auth/me` con/sin sesión, logout 204, refresh fallido → 401, CSRF 403/válido.
3. **Bloque 3 — Validación de tokens y autorización** (tasks 3.1–3.5). Tests: kid desconocido, caducado, `aud` ajeno, `alg=none`; 401/403/200 por permiso; públicos; CORS por `ALLOWED_ORIGINS`; cookie o Bearer.
4. **Bloque 4 — Agent API M2M** (tasks 4.1–4.4). Tests: M2M con `agent:*` accede; usuario sin `agent:*` → 403; contrato sin cambios; retirada del API key (solo doc, no existe en código).
5. **Bloque 5 — Frontend** (tasks 5.1–5.5). Tests (vitest + Testing Library): estado de carga, guard de rutas con destino, 401→re-login, 403→aviso sin destruir sesión, `X-CSRF-Token` en mutaciones, nombre + logout.
6. **Bloque 6 — IdP de desarrollo** (tasks 6.1–6.4). `compose` con Pocket ID; guía de API/resource `https://oximeal.local/api`, keys/permisos, cliente user-delegated (*Allowed user groups*), cliente M2M (Valet), HTTPS por reverse proxy.
7. **Bloque 7 — Tests y CI** (tasks 7.1–7.7). Puertas: `cargo test` (con y sin `embed-ui`), `cargo test` con `OIDC_ENABLED=false`, `clippy -D warnings`, `fmt --check`, `just ci`, `just ci-ui`, `openspec validate oidc-auth --strict`.

Bloque 8 (docs: `.env.example`, README, permisos, migración del API key, dependencia Fase 5) se cierra junto con el bloque 7.

### 12.8 Estrategia de testing

- **Unit**: `config.rs` (validación), `jwt.rs` (claims), `csrf.rs` (comparación), mapeo de permisos.
- **Integración** (`backend/tests/*`): mock IdP con **`wiremock`** (discovery, JWKS y token endpoint) y tokens firmados con una clave RSA de test estática (`jsonwebtoken::encode`). Se ejercita el flujo BFF completo con cookies (`Set-Cookie`/`Cookie`) y `Authorization: Bearer`.
- **Frontend**: `vitest` + Testing Library, mockeando `src/api/client` y el proveedor de sesión.
- **IdP caído**: el arranque y `GET /api/health` no dependen del IdP; rutas protegidas sin credencial → 401.

### 12.9 Riesgos específicos de la ejecución

| Riesgo | Mitigación |
|---|---|
| Incompatibilidad de versión `tower-sessions` ↔ axum 0.8 | Verificar versiones con `cargo add`/docs antes de fijar; alternativa: `SessionStore` mínimo propio. |
| Cripto de test (clave RSA) frágil | Clave PEM estática y determinista en `tests/common`; nunca en producción. |
| Consultar el IdP real en tests | Prohibido: siempre mock (wiremock) o `OIDC_ENABLED=false`. |
| Fuga de tokens por logs (riesgo global) | Prohibido loguear `Cookie`/`Authorization`; `/auth/me` nunca devuelve tokens. |
| `OIDC_ENABLED=false` rompe tests existentes | `Config` por defecto en tests sin OIDC; suite legacy debe seguir verde. |

## 13. Hardening de SQLite (issues #2 y #4) — change `sqlite-hardening`

Capability nueva `persistence` en OpenSpec. Estado: implementado, en PR a `development`.

### 13.1 Objetivo

- **#2**: eliminar `SQLITE_BUSY_SNAPSHOT` en las operaciones read-then-write abriendo las transacciones de escritura con `SqlitePool::begin_with("BEGIN IMMEDIATE")` (sqlx 0.9; `&'static str` implementa `SqlSafeStr`).
- **#4**: garantizar integridad referencial y de dominio con `ON DELETE RESTRICT` en `meal_plan_entries.recipe_id → recipes(id)` y `CHECK` de dominio.

### 13.2 Decisiones de diseño

- Un único change `sqlite-hardening` cubre #2 y #4.
- Escrituras con `begin_with("BEGIN IMMEDIATE")`; lecturas siguen con `begin()` (diferido) para no serializarlas.
- `ON DELETE RESTRICT` (no `CASCADE`) para impedir borrar recetas referenciadas por el plan.
- `CHECK` de dominio: `shopping_list_items.amount >= 0`; `recipes.servings` y `meal_plan_entries.servings` NULL o `>= 1`; `recipes.prep_time_minutes` NULL o `>= 0`.

### 13.3 Módulos y migración

- `backend/src/repo/agent.rs` (`set_weekly_plan`, `add_shopping_items`) y `backend/src/repo/shopping_list.rs` (`consolidate`): las 3 transacciones de escritura usan `begin_with`.
- Migración nueva append-only `0005_schema_constraints.sql`: rebuild de 12 pasos de `recipes` (+ recrear `recipes_fts`, triggers e `INSERT INTO recipes_fts(recipes_fts) VALUES('rebuild')`), `meal_plan_entries` y `shopping_list_items`. Primera línea `-- no-transaction` para poder usar `PRAGMA foreign_keys = OFF` fuera de transacción; el DDL va envuelto en `BEGIN/COMMIT` (atómico) con `PRAGMA foreign_key_check` antes del `COMMIT`.

### 13.4 Tests

- `backend/tests/write_transactions.rs` (4): reproducción de `BUSY_SNAPSHOT` con `BEGIN` diferido (código extendido `517`), serialización con `BEGIN IMMEDIATE`, y concurrencia de `set_weekly_plan` + `consolidate` + `add_shopping_items` sin `BUSY_SNAPSHOT`.
- `backend/tests/schema_constraints.rs` (9): FK `on_delete == 'RESTRICT'` vía `PRAGMA foreign_key_list`; borrado de receta referenciada rechazado; `CHECK` de amount/servings/prep_time (negativos y `< 1` rechazados, válidos y NULL aceptados); datos y búsqueda FTS5 sobreviven a la migración.

### 13.5 Criterios de aceptación (DoD)

- [x] `cargo test` verde (454 tests con `embed-ui`); `clippy --all-targets --all-features -- -D warnings` limpio; `fmt --check` limpio.
- [x] `just ci` exit 0 y `just ci-ui` exit 0.
- [x] `openspec validate --all --strict` → 17/17.
- [x] Issues #2 y #4 cerradas (PR #19 fusionado en `development`); change `sqlite-hardening` archivado.

## 14. Hardening del servido estático (issues #8 y #9) — change `static-serving-hardening`

Capability `frontend-serving` (requisitos añadidos). Estado: implementado, en PR a `development`.

### 14.1 Objetivo

- **#8**: revalidación con `ETag`/`If-None-Match` → `304 Not Modified` sin cuerpo, en vez de un `200` con cuerpo completo en cada revalidación.
- **#9**: `charset=utf-8` en tipos `text/*` y cabeceras de seguridad (`X-Content-Type-Options: nosniff` y `Content-Security-Policy`).

### 14.2 Decisiones de diseño

- `ETag` fuerte `"<hex sha256>"` tomado del hash que `rust-embed` calcula en compilación (`EmbeddedFile::metadata.sha256_hash()`), sin re-hashear activos grandes por petición.
- `If-None-Match` según RFC 9110 (lista separada por comas, `W/`, comodín `*`).
- CSP compatible con Ant Design CSS-in-JS (`style-src 'self' 'unsafe-inline'`), aplicada solo a documentos HTML y XHTML; `nosniff` en todas las respuestas estáticas.
- `Range` **no soportado** por decisión (activos inmutables y pequeños de un SPA): `200` completo, sin `Accept-Ranges`.

### 14.3 Módulos

- `backend/src/static_files/embedded.rs`: `serve(method, headers, uri)` + helpers `etag_value`, `content_type_for`, `cache_control_for`, `apply_security_headers`, `with_security_headers`, `if_none_match_matches`, `respond`, `asset_response`.
- Tests: `backend/tests/static_files.rs` (23 integración con `embed-ui`) + unitarios en `embedded.rs`.

### 14.4 Criterios de aceptación (DoD)

- [x] `just ci` exit 0 (467 tests con `embed-ui`) y `just ci-ui` exit 0.
- [x] `clippy -D warnings` y `fmt --check` limpios.
- [x] `openspec validate --all --strict` en verde.
- [x] Issues #8 y #9 cerradas; change `static-serving-hardening` archivado.

## 15. Consolidación escalada por raciones (issue #3) — change `consolidation-servings-scaling`

Capability `ingredient-consolidation` (requisito añadido). Estado: implementado, PR #23 fusionado.

### 15.1 Objetivo

Al consolidar la lista de la compra, escalar la cantidad de cada ingrediente por el cociente entre las raciones planificadas en la entrada del plan y las raciones de la receta, de modo que una receta de 2 raciones planificada a 4 duplique las cantidades.

### 15.2 Decisión de diseño

- El escalado se hace **en `collect_drafts`** (`backend/src/repo/shopping_list.rs`), **antes** de `domain::consolidate`: la consulta lee `COALESCE(e.servings, 2)` y `COALESCE(r.servings, 2)` y multiplica cada `IngredientInput.amount` por `factor = entry / recipe`. El motor de consolidación no cambia.
- Dos rutas afectadas: `POST /api/shopping-list/consolidate` y `POST /api/agent/weekly-plan` (ambas comparten `collect_drafts` y `BEGIN IMMEDIATE`).
- Helper puro `servings_factor(entry, recipe)` con denominador acotado (`recipe.max(1)`) por defensa; `round2` redondea una sola vez al final por subgrupo.

### 15.3 Tests

- `backend/tests/shopping_list.rs`: 5 de integración (2→4 duplica; 4→2 mitad; sin raciones factor 1; `NULL`/`NULL` factor 1; misma receta con raciones distintas se suma) + unitario de `servings_factor`.

### 15.4 Criterios de aceptación (DoD)

- [x] `just ci` exit 0 (481 tests) y `just ci-ui` exit 0.
- [x] `clippy -D warnings` y `fmt --check` limpios.
- [x] `openspec validate --all --strict` en verde.
- [x] Issue #3 cerrada (PR #23); change `consolidation-servings-scaling` archivado.

## 16. Semántica de búsqueda trigram (issue #6) — change `trigram-search-semantics`

Capability `recipe-search` (requisito documental añadido). Estado: implementado, PR #24 fusionado.

### 16.1 Objetivo

Documentar y fijar con tests de caracterización la semántica real del buscador de recetas: `q` se envía entre comillas (frase) a `recipes_fts MATCH` y, con el tokenizer `trigram`, equivale a una **subcadena contigua** del nombre o de las instrucciones, no a un AND de palabras.

### 16.2 Hallazgo / decisión

- Con `q` de **≥3 caracteres**: coincidencia por subcadena contigua, insensible a mayúsculas y diacríticos; `"estofadas con"` coincide, `"lentejas zanahoria"` no; `"cafe"` encuentra `"Café"`.
- Con `q` de **<3 caracteres**: fallback `LIKE '%q%'` **solo sobre el nombre** (no pliega diacríticos; case-insensitive solo ASCII).
- Sin cambio de comportamiento de producción: solo docstring de `repo::recipes::list`, `docs/search.md` (nuevo) y tests.

### 16.3 Tests

- `backend/tests/recipes.rs`: 5 de caracterización (subcadena contigua multi-palabra; palabras no contiguas → `total=0`; término de 3 chars solo en instrucciones → FTS; `<3` chars no busca en instrucciones).

### 16.4 Criterios de aceptación (DoD)

- [x] `just ci` exit 0 (489 tests) y `just ci-ui` exit 0.
- [x] `clippy -D warnings` y `fmt --check` limpios.
- [x] `openspec validate --all --strict` en verde.
- [x] Issue #6 cerrada (PR #24); change `trigram-search-semantics` archivado.

## 17. Frontend hardening: a11y/Prettier, lazy antd y haptics (issues #11, #13, #10)

Estado: planificado. Tres changes independientes, una rama/PR por issue desde `development`.

### 17.1 Objetivo

Cerrar los tres hallazgos de la Fase 2 del frontend sin cambiar el comportamiento de producto:
- #11: linting estático de accesibilidad y formato verificable en CI.
- #13: sacar antd del critical path de arranque mediante carga diferida.
- #10: alinear spec y código de haptics documentando la Vibration API.

### 17.2 Issue #11 — `a11y-lint-prettier`

**Decisión:** añadir `eslint-plugin-jsx-a11y@^6.10.2`, `prettier@^3.9.10` y `eslint-config-prettier@^10.1.8`. El peer de `eslint-plugin-jsx-a11y` declara ESLint ≤9; se resuelve con `pnpm.peerDependencyRules.allowedVersions` para ESLint 10 y se valida que las reglas funcionan.

**Cambios:**
- `frontend/eslint.config.js`: `jsxA11y.flatConfigs.recommended` + `eslint-config-prettier` al final.
- `frontend/.prettierrc.json` (`singleQuote`, `trailingComma: all`, `printWidth: 100`, `semi`) y `.prettierignore` (`dist`, `node_modules`, `coverage`, `pnpm-lock.yaml`, `*.md`, `tsconfig.tsbuildinfo`).
- `frontend/package.json`: scripts `format` y `format:check`; bloque `pnpm.peerDependencyRules`.
- `justfile` (`ci-ui`) y `.github/workflows/ci.yml`: paso `pnpm format:check`.
- Formatear `frontend/src` y corregir los avisos de a11y que aparezcan.

**Tests/verificación:** `pnpm lint` 0 problemas, `pnpm format:check` limpio, `pnpm typecheck`, `pnpm test`, `pnpm build` y `just ci-ui` en verde.

**DoD:** reglas de a11y activas; Prettier integrado en `just ci-ui` y CI; change archivado; issue #11 cerrada.

- [x] `pnpm lint` 0 problemas y `pnpm format:check` limpio (110 tests).
- [x] `just ci-ui` exit 0; `openspec validate --all --strict` en verde (20/20).
- [x] Versiones: `eslint-plugin-jsx-a11y@6.10.2`, `prettier@3.9.10`, `eslint-config-prettier@10.1.8`; peer de ESLint 10 resuelto en `pnpm-workspace.yaml`.
- [x] Change `2026-11-02-a11y-lint-prettier` archivado; capability `frontend-tooling` creada.

### 17.3 Issue #13 — `antd-lazy-boot`

**Decisión:** mantener los componentes antd actuales y cargar todo el árbol que los usa de forma diferida. No se reescribe el shell ni las páginas.

**Cambios:**
- `frontend/src/AppRoot.tsx` (nuevo, default export): `ErrorBoundary`, `ConfigProvider`, `AntdApp`, `ForbiddenNotifier`, `BrowserRouter` y `App`.
- `frontend/src/components/BootFallback.tsx` y `BootErrorBoundary.tsx` (nuevos, sin antd): fallback de primer pintado y boundary de arranque.
- `frontend/src/main.tsx`: `lazy(() => import('./AppRoot'))` + `Suspense` + `BootErrorBoundary`; se mantienen `QueryClientProvider` y `SessionProvider` en el entry.
- `frontend/src/api/client.ts`: eliminar `import { message } from 'antd'`; el fallback de 403 usa `void import('antd').then(({ message }) => message.warning(...))`.
- `frontend/vite.config.ts`: `build.manifest = true`.
- `frontend/scripts/check-bundle.mjs` (nuevo): recorre el manifest y suma el grafo estático del entry; falla si supera 500 kB en bruto.
- `justfile`: `ci-ui` ejecuta `node scripts/check-bundle.mjs` tras el build.

**Medición:** línea base `antd` 694,20 kB (224,66 kB gzip), entry 11,23 kB. Objetivo: el grafo estático inicial (entry + react + query + session/client) < 500 kB en bruto; antd pasa a chunk asíncrono. Se anotan los tamaños antes/después en este apartado y en `CHANGELOG.md`.

**Tests/verificación:** `pnpm build` + `check-bundle` en verde; `pnpm typecheck`, `pnpm test`, `pnpm lint`; prueba manual de shell, login, menú y compra.

**DoD:** antd fuera del grafo estático inicial; presupuesto < 500 kB; UI sin regresiones; change archivado; issue #13 cerrada.

- [x] Antes: grafo estático del entry 761,3 kB (antd 694,20 kB / 224,66 kB gzip en el chunk estático).
- [x] Después: grafo estático del entry **293,7 kB** (react 251,9 + query 34,0 + entry 7,1 + runtime 0,6); antd pasa a chunk diferido de 458,95 kB (150,86 kB gzip).
- [x] `pnpm lint` 0 problemas, 114 tests; `pnpm check-bundle` (presupuesto 500 kB) y `just ci-ui` en verde.
- [x] `openspec validate --all --strict` en verde (19/19 con @fission-ai/openspec@1.16.0); change `2026-11-02-antd-lazy-boot` archivado.

### 17.4 Issue #10 — `haptics-vibration-api`

**Decisión:** documentar la Vibration API (`navigator.vibrate`) y no migrar a `web-haptics` (0.0.6, API inmadura, dependencia nueva y workaround frágil para iOS). En navegadores sin soporte (iOS Safari) el hook es un no-op.

**Cambios:**
- `openspec/specs/shopping-mode/spec.md`: requisito "Marcar como comprado" pasa de "Web Haptics" a "Vibration API (`navigator.vibrate`)" con no-op si no hay soporte.
- `frontend/src/hooks/useHaptics.ts`: docstring con la decisión y el comportamiento en iOS Safari.
- `frontend/src/hooks/useHaptics.test.ts`: tests del patrón por defecto (10 ms) y de patrón en array.

**Tests/verificación:** `pnpm test`, `pnpm typecheck`, `pnpm lint`; `openspec validate --all --strict`.

**DoD:** spec y docstring coherentes; tests en verde; change archivado; issue #10 cerrada.

- [x] Spec `shopping-mode` (requisito y Purpose) alineada con la Vibration API (`navigator.vibrate`); docstring de `useHaptics` documenta la decisión y el no-op en iOS Safari.
- [x] Tests del patrón por defecto (10 ms) y de patrón en array; `pnpm test` (116), `pnpm typecheck` y `pnpm lint` (0 problemas) en verde.
- [x] `openspec validate --all --strict` en verde (18/18 con @fission-ai/openspec@1.16.0); change `2026-11-02-haptics-vibration-api` archivado.

### 17.5 Orden de ejecución y ramas

1. `feature/a11y-lint-prettier` → PR a `development`.
2. `feature/antd-lazy-boot` → PR a `development`.
3. `feature/haptics-vibration-api` → PR a `development`.

Cada PR: `just ci-ui` + `openspec validate --all --strict` en verde, `CHANGELOG.md` actualizado y change archivado tras el merge.
## 18. Fase 5 — PWA/Offline (shell instalable, caché de datos y cola de escrituras)

Se ejecuta en **tres changes encadenados**, cada uno en su rama y su PR a `development`. El Service Worker es propio (vanilla), sin Workbox ni `vite-plugin-pwa`; los iconos se generan con IA y se derivan los tamaños con `magick`.

### 18.1 Objetivo

Convertir OxiMeal en una PWA instalable y offline-first: (a) shell instalable (manifiesto + iconos + Service Worker de activos), (b) lectura sin conexión con tolerancia a la sesión OIDC caducada, y (c) escritura sin conexión con cola de sincronización. Cierra la Fase 5 del roadmap (§6) y la dependencia registrada en §12/§4 (la Fase 5 debe tolerar sesión caducada sirviendo datos cacheados).

### 18.2 Decisiones de diseño (ADR-18…27)

- **ADR-18 — Service Worker propio.** `frontend/public/sw.js`, sin dependencias de build. Coherente con la CSP existente (`worker-src 'self' blob:`) y con el ethos de binario único. Alternativa descartada: `vite-plugin-pwa`/Workbox (añade dependencias de build).
- **ADR-19 — El SW NO intercepta `/api/*` ni `/auth/*`.** La caché de datos vive en IndexedDB (capa de datos), no en el HTTP cache del SW. Evita cachear respuestas autenticadas, fugas de datos entre sesiones y respuestas 401 cacheadas.
- **ADR-20 — Estrategias del SW.** `install`: precache del shell y de los assets con hash (inyectados en build). `activate`: limpieza de cachés `oximeal-*` obsoletas y `clients.claim()`. `fetch`: *cache-first* para `assets/*` (nombres con hash, inmutables), *network-first* para navegaciones con fallback al `index.html` cacheado, *stale-while-revalidate* acotado a manifiesto, favicon e iconos. Sin `skipWaiting()`: el SW nuevo espera a que se cierren las pestañas para no romper chunks de un build anterior.
- **ADR-21 — Precache y versión inyectados en build.** `frontend/scripts/build-sw.mjs` lee `.vite/manifest.json`, compone la lista de assets (entry, chunks diferidos y CSS) y la inyecta en `dist/sw.js` junto con una `CACHE_VERSION` derivada del hash de esa lista. El script falla si no encuentra los marcadores, si el precache queda vacío o si falta algún asset en `dist`.
- **ADR-22 — Registro solo en producción.** El SW se registra desde `frontend/src/offline/register-sw.ts` únicamente si `import.meta.env.PROD`; en `pnpm dev` no se registra (evita interferencias con HMR).
- **ADR-23 — Manifiesto e iconos en `public/`.** `frontend/public/` no existía; se crea. Al vivir en `public/`, el manifiesto y el SW se copian verbatim a `dist/` y **no** entran en el grafo del bundle (el presupuesto de 500 kB de §17.3 sigue intacto).
- **ADR-24 — Caché de datos con IndexedDB propio.** Wrapper tipado sobre la API nativa (`frontend/src/offline/idb.ts`), DB `oximeal` con stores `queryCache` y `mutations`. Cero dependencias nuevas.
- **ADR-25 — Persistencia de react-query sin librería extra.** Hidratación al arranque (antes del primer render de datos) y persistencia tras cada carga correcta usando `dehydrate`/`hydrate` de `@tanstack/react-query` y una suscripción a `queryClient.getQueryCache()`, con lista blanca de claves (`shopping-list`, `meal-plan`, `recipes`). Se descarta `@tanstack/react-query-persist-client` para no añadir dependencias.
- **ADR-26 — Modo degradado por sesión caducada.** Si hay caché y no hay red (o la sesión ha caducado estando offline), la aplicación muestra los datos cacheados con un aviso persistente y **no** redirige al login ni vacía la vista. Si hay red y 401, se mantiene el comportamiento actual (volver al login).
- **ADR-27 — Cola de escrituras FIFO con idempotencia por `client_id`.** Las mutaciones offline se encolan en IndexedDB y se drenan al recuperar la conexión, en orden. Para que un alta reenviada tras un fallo de red no duplique ítems, el backend acepta un `client_id` opcional y único en `POST /api/shopping-list` (devuelve el ítem existente si ya existe). Reconciliación last-write-wins por `updated_at`; un 4xx no recuperable descarta la operación y notifica; un 401 o un fallo de red detiene el drenado.

### 18.3 Change A — `pwa-installable-shell` (issue #29)

**Cambios:**
- `frontend/public/manifest.webmanifest`: `id`, `name`, `short_name`, `description`, `start_url: "/"`, `scope: "/"`, `display: "standalone"`, `orientation: "any"`, `theme_color: "#2f855a"`, `background_color: "#ffffff"`, `lang: "es"`, `icons` (192, 512, 512 maskable).
- `frontend/public/icons/`: `icon-192.png`, `icon-512.png`, `icon-512-maskable.png`, `apple-touch-icon-180.png` (derivados del icono maestro con `magick`), más `favicon.svg`.
- `frontend/public/sw.js`: Service Worker de activos según ADR-20.
- `frontend/scripts/build-sw.mjs`: inyección de precache y versión (ADR-21); `package.json` lo ejecuta tras `vite build`.
- `frontend/src/offline/register-sw.ts`: registro en producción; invocado desde `frontend/src/main.tsx`.
- `frontend/index.html`: `<link rel="manifest">`, `<link rel="icon">`, `<link rel="apple-touch-icon">`.
- `backend/src/static_files/embedded.rs`: mapeo explícito `.webmanifest` → `application/manifest+json` (independiente de `mime_guess`), `.webmanifest` tratado como fichero en `is_orphan_asset`, y `Cache-Control: no-cache` para `sw.js`.

**Tests/verificación:** 137 tests en frontend (arnés real del SW con `caches`/`self`/`fetch` simulados y ciclo de vida fiel al evento); 271 tests en backend con `embed-ui` (tests puros de `content_type_for`/`cache_control_for`/`is_orphan_asset` + e2e con guard, verdes también sin `frontend/dist`).

**DoD:** manifiesto e iconos servidos con MIME correcto; SW registrado solo en producción; recarga offline del shell garantizada por el precache; CSP sin cambios; change archivado; issue #29 cerrada.

- [x] Manifiesto `manifest.webmanifest` (`id`, `scope`, `display: standalone`, `orientation: any`) e iconos 192/512/maskable/apple-touch-180 derivados del icono maestro con `magick`.
- [x] `frontend/public/sw.js`: precache del shell + assets con hash inyectados en build (16 assets, versión = hash del listado); `cache-first` para `assets/*`, `network-first` para navegaciones con fallback al `index.html`, `stale-while-revalidate` acotado a manifiesto/favicon/iconos; sin `skipWaiting`; limpieza acotada al prefijo `oximeal-`; no intercepta `/api` ni `/auth`.
- [x] Registro del Service Worker solo en producción (`register-sw.ts`), con `readyState` y `console.warn` en el rechazo.
- [x] Backend: `.webmanifest` → `application/manifest+json` con mapeo explícito independiente de `mime_guess`; `sw.js` con `Cache-Control: no-cache`; `.webmanifest` tratado como fichero (404, no fallback SPA).
- [x] `just ci`, `just ci-ui` (294,0 kB / 500 kB) y `openspec validate --all --strict` (21/21) en verde.

### 18.4 Change B — `offline-data-cache` (issue #30)

**Cambios:**
- `frontend/src/offline/idb.ts`: wrapper tipado de IndexedDB (DB `oximeal`; stores `queryCache` y `mutations`).
- `frontend/src/offline/queryPersist.ts`: hidratación y persistencia del `QueryClient` (ADR-25).
- `frontend/src/offline/connectivity.ts`: hook de estado de red (`useOnline`) sobre eventos `online`/`offline`.
- `frontend/src/auth/SessionProvider.tsx` y `frontend/src/auth/AuthGuard.tsx`: modo degradado (ADR-26); limpieza de la caché de IndexedDB al cerrar sesión.
- `frontend/src/components/OfflineBanner.tsx`: aviso de modo degradado.
- `frontend/src/main.tsx`: hidratar la caché antes del primer render de datos.

**Tests/verificación:** arranque offline con caché previa muestra la lista guardada; la persistencia se escribe tras una carga correcta; 401 sin red no redirige; 401 con red redirige; el logout limpia IndexedDB.

**DoD:** datos disponibles sin conexión; modo degradado visible; logout limpia la caché; change archivado; issue #30 cerrada.

- [x] Issue #30. `frontend/src/offline/idb.ts`: wrapper tipado de IndexedDB (DB `oximeal`, store `queryCache`) con interfaz `OfflineStore` inyectable, `replaceQueries` transaccional, `onversionchange`/`onabort` y degradación silenciosa sin `indexedDB`.
- [x] `frontend/src/offline/queryPersist.ts`: hidratación con `hydrate`/`dehydrate` de react-query, persistencia con debounce de 250 ms + `visibilitychange`/`pagehide`, lista blanca (`shopping-list`, `meal-plan`, `recipes`), poda de claves obsoletas y flag `hasCachedData` sincronizado.
- [x] `frontend/src/offline/connectivity.ts`: `isOnline()` y `useOnline()` con `useSyncExternalStore`.
- [x] Modo degradado: `SessionStatus` gana `degraded`; 401 sin conexión con caché no redirige; 401 con conexión limpia `QueryClient` e IndexedDB antes de ir al login; la reconexión revalida la sesión.
- [x] `OfflineBanner` en `AppLayout`; `main.tsx` hidrata antes del primer render con timeout de 2 s para no bloquear el arranque.
- [x] Tests: 165 en frontend; `just ci-ui` (300,0 kB / 500 kB) y `openspec validate --all --strict` (21/21) en verde.

### 18.5 Change C — `offline-write-queue` (issue #31)

**Cambios backend:**
- Migración `0006_shopping_item_client_id.sql`: columna `client_id TEXT` con índice único parcial.
- `backend/src/models` y `backend/src/repo/shopping_list.rs`: soporte de `client_id`.
- `backend/src/routes/shopping_list.rs`: `POST /api/shopping-list` idempotente por `client_id`.

**Cambios frontend:**
- `frontend/src/offline/mutationQueue.ts`: cola FIFO en IndexedDB, drenado al volver `online`, reconciliación de `client_id` provisional → real, LWW por `updated_at`.
- `frontend/src/api/hooks.ts`: mutaciones offline-aware (encolar en vez de fallar).
- `frontend/src/pages/ShoppingPage.tsx`: indicador de cambios pendientes de sincronizar.

**Tests/verificación:** alta offline aparece localmente y se sincroniza al volver la conexión; reenvío con el mismo `client_id` no duplica (backend); toggle de un ítem no sincronizado se aplica a la operación encolada; 4xx no recuperable descarta y notifica.

**DoD:** escritura offline funcional; alta idempotente; sin duplicados tras reconexión; change archivado; issue #31 cerrada.

- [x] Issue #31. Backend: migración `0006_shopping_item_client_id.sql` (columna `client_id TEXT` + índice único parcial), `client_id` en modelo y repositorio, y `POST /api/shopping-list` idempotente (201 al crear, 200 con el ítem existente); el `SELECT` de `repo/agent.rs` incluye la columna nueva.
- [x] `frontend/src/offline/mutationStore.ts`: store `mutationQueue` en IndexedDB (DB compartida `oximeal` v2, `keyPath: seq` autoincrement) con interfaz inyectable y degradación a cola en memoria.
- [x] `frontend/src/offline/mutationQueue.ts`: cola FIFO, reentrante, drenado con parada en red/401 y descarte con aviso en 4xx no recuperable, reconciliación de ids `local:*` → ids reales, fusión de toggles sobre altas aún no sincronizadas y LWW con `parseServerTimestamp` (UTC real, no hora local).
- [x] `frontend/src/api/hooks.ts`: mutaciones offline-aware (`useAddShoppingItem` encola en vez de fallar y cae al camino offline ante error de red; `useToggleShoppingItem` no llama al servidor si el ítem es local o no hay conexión, con rollback decidido por el contexto de `onMutate`); `networkMode: 'offlineFirst'` en las consultas.
- [x] `syncShoppingQueue.ts` + `MutationQueueProvider` + `MutationQueueNotifier` (dentro de `<AntdApp>`, con `App.useApp()`); aviso de pendientes con botón «Sincronizar ahora» en `ShoppingPage` y etiqueta `Pendiente` en `ShoppingRow`.
- [x] Tests: 199 en frontend (5 ejecuciones seguidas en verde, sin flakiness); `just ci`, `just ci-ui` (309,2 kB / 500 kB) y `openspec validate --all --strict` (21/21) en verde.

### 18.6 Orden de ejecución y ramas

1. `feature/pwa-installable-shell` → PR a `development` (issue #29).
2. `feature/offline-data-cache` → PR a `development` (issue #30).
3. `feature/offline-write-queue` → PR a `development` (issue #31).

Cada PR: `just ci` + `just ci-ui` + `openspec validate --all --strict` en verde, `CHANGELOG.md` actualizado y change archivado tras el merge.

### 18.7 Riesgos

| Riesgo | Mitigación |
|---|---|
| Doble caché (SW y datos) con estados divergentes | El SW no toca `/api/*` ni `/auth/*` (ADR-19): activos y datos tienen dueños distintos. |
| Fuga de datos entre usuarios en IndexedDB | Limpieza de la caché de datos al cerrar sesión (Change B). |
| Complejidad de la cola de escrituras | Acotada a 4 operaciones, FIFO, idempotencia por `client_id` y LWW; 4xx no recuperable se descarta. |
| Cambio de contrato del alta de ítems | `client_id` es **opcional** y retrocompatible; la ruta sigue creando sin él. |
| SW sirviendo un shell obsoleto o rompiendo pestañas abiertas | `no-cache` en `sw.js`, versión derivada del build, sin `skipWaiting` y limpieza acotada por prefijo. |
| Precache silenciosamente vacío | `build-sw.mjs` falla si no inyecta, si el precache queda vacío o si falta un asset en `dist`. |
| Presupuesto de bundle | Manifiesto e iconos viven en `public/`, fuera del grafo del entry. |

### 18.8 Criterios de aceptación (DoD de la Fase 5)

- [x] Aplicación instalable en Android (Chrome) e iOS (Safari) con iconos correctos.
- [x] Shell arranca sin conexión tras una visita previa.
- [x] Lista de la compra visible sin conexión y con sesión caducada, con aviso de modo degradado.
- [x] Mutaciones offline encoladas y sincronizadas sin duplicados al recuperar la conexión.
- [x] `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde en las tres ramas.
- [x] Los tres changes archivados y la Fase 5 marcada como cerrada en este plan.

> Verificado el 2026-11-02 con Chrome sobre el binario con `--features embed-ui`
> y `OIDC_ENABLED=false`: smoke E2E **37/37**. Chrome reporta 0 errores de
> instalabilidad (`Page.getInstallabilityErrors` → `[]`); el shell arranca sin
> conexión; la lista se ve desde la caché con aviso de modo degradado; el alta
> offline se encola, sobrevive a la recarga y se sincroniza sola al recuperar la
> conexión con **un único POST** y sin duplicados; reenviar el mismo `client_id`
> responde 200 sin duplicar.

## 19. Fase 6 — Recetario completo y verificación en dispositivo real

> Ejecutable ahora. Origen: issues #45, #46 y #47. Tres changes independientes,
> una rama/PR por issue desde `development`. Metodología SDD + TDD estricto
> (RED → GREEN → REFACTOR), con auditoría por bloque.

### 19.1 Objetivo

Completar el **recetario** (hoy la API de recetas es solo listar/buscar/ver y
crear, y el frontend **no tiene página de recetas**: solo `MenuPage` y
`ShoppingPage`), permitir **importar una receta desde una URL**, y cerrar la
**verificación de rendimiento y consumo de RAM** en dispositivo real pendiente
de la Fase 5 (pruebas táctiles y de rendimiento) y del objetivo global `<20 MB`
registrado en §10.

### 19.2 Decisiones de diseño (ADR-28…31)

| # | Decisión | Justificación |
|---|----------|---------------|
| ADR-28 | Actualización de receta por **reemplazo total** (`PUT /api/recipes/{id}`) en una transacción `BEGIN IMMEDIATE`, reescribiendo ingredientes y confiando en los triggers `ai/ad/au` para refrescar el índice FTS | Atomicidad y coherencia con la consolidación; sin parches incrementales frágiles sobre ingredientes |
| ADR-29 | Borrado **duro** de receta (`DELETE /api/recipes/{id}`) con borrado en cascada de ingredientes (FKs `ON DELETE`) y limpieza del índice FTS por trigger | Sin estado «borrado lógico» que complicaría la búsqueda; el borrado físico es lo esperado en self-hosted |
| ADR-30 | Importación **server-side** con `reqwest` (ya presente para OIDC) y guardas **SSRF**: solo `http`/`https`, bloqueo de rangos privados/loopback/link-local tras resolución DNS, límite de redirecciones, *timeout* y tope de tamaño; parseo de JSON-LD `Recipe` de Schema.org; deduplicación por `source_url` | El navegador no puede leer el HTML de terceros (CORS) y centralizar la descarga permite validar y proteger |
| ADR-31 | Verificación de rendimiento como **medición reproducible on-demand** (receta `just`), no como *gate* de CI; presupuestos web documentados | Las métricas de rendimiento en CI son ruidosas y frágiles; se miden y documentan bajo demanda |

### 19.3 Change A — `recipes-management` (issue #45)

- **Backend:** `PUT /api/recipes/{id}` y `DELETE /api/recipes/{id}` (permiso
  `write:recipes`), con validación reutilizada y `404` cuando no existe;
  transacción `BEGIN IMMEDIATE` para el reemplazo total; OpenAPI actualizado.
- **Frontend:** nueva `RecipesPage` (`/recipes`) + detalle (`/recipes/:id`) con
  lista, búsqueda (hooks `useRecipes`/`useRecipe` ya existentes), alta, edición y
  borrado con confirmación; entrada de navegación en `AppLayout` (Sider) y
  `BottomNav` (🍳 Recetas); objetivos táctiles ≥48px.
- **Specs:** delta en `recipes` (ADDED *Actualización* y *Borrado*), capacidad
  nueva `recipe-catalog` (UI) y delta en `ui-shell` (ADDED *Pestaña Recetas*).
- **Estado:** ✅ completado e integrado en el PR #49 (smoke E2E 23/23, `just ci`/`just ci-ui` y `openspec validate --all --strict` en verde).

### 19.4 Change B — `recipe-import-schema-org` (issue #46)

- **Backend:** `POST /api/recipes/import` con cuerpo `{ "url": "..." }` (permiso
  `write:recipes`): descarga con guardas SSRF (ADR-30), extracción del bloque
  JSON-LD `application/ld+json` con `@type: Recipe`, mapeo a `RecipeInput`
  (nombre, ingredientes, `recipeYield`→`servings`, `totalTime`/`prepTime`→minutos,
  `recipeInstructions`, `source_url`), alta **idempotente** por `source_url`.
  Errores tipados: `400` (URL inválida/prohibida), `422` (sin receta reconocible),
  `502` (fallo de origen), `409` (ya importada) o `200` con la existente.
- **Frontend:** diálogo «Importar desde URL» en `RecipesPage` con previsualización
  antes de guardar y manejo de errores.
- **Specs:** delta en `recipes` (ADDED *Importación de receta desde URL*), con
  escenarios de SSRF, contenido sin receta y fallo de red.
- **Estado:** ✅ completado e integrado en el PR #50 (smoke E2E 18/18, `just ci`/`just ci-ui` y `openspec validate --all --strict` en verde).

### 19.5 Change C — `perf-device-verification` (issue #47)

- **Medición de RAM:** receta `just` que arranca `cargo build --release -p
  oximeal-backend --features embed-ui`, lanza el binario con una base temporal y
  mide RSS **pico y estable** (varias muestras), dejando el número documentado.
- **Rendimiento web:** auditoría por **CDP de Chrome headless, sin
  dependencias** (FCP y transferencia del grafo inicial declarado; no se miden
  LCP ni TTI) con presupuestos explícitos; comprobación táctil de objetivos
  ≥48px, trazada en `docs/performance.md` junto con los escenarios de
  interacción con/sin conexión del smoke E2E de Fase 5.
- **Entregable:** `docs/performance.md` con metodología, presupuestos y
  resultados, y la receta `just` reproducible.
- **Specs:** capacidad nueva `performance` (ADDED *Presupuesto de memoria*,
  *Presupuesto de rendimiento web*, *Verificación táctil*).

### 19.6 Orden de ejecución y ramas

1. `feature/recipes-management` (issue #45) desde `development`.
2. `feature/recipe-import-schema-org` (issue #46), sobre el anterior ya mergeado.
3. `feature/perf-device-verification` (issue #47), el último (mide con la UI ya
   completa).

### 19.7 Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| Import vulnerable a SSRF | Guardas en ADR-30 + tests con `wiremock` y URLs a `127.0.0.1`/`localhost`/rangos privados |
| `PUT` desincroniza el índice FTS | Triggers `ai/ad/au` existentes + test de búsqueda tras editar |
| Medición de RAM ruidosa | Medir varias muestras, reportar pico y estable, documentar el método |
| Web scraping de terceros frágil | Se documenta que el import depende de JSON-LD estándar; fallo → `422` claro |

### 19.8 Criterios de aceptación (DoD de la Fase 6)

- [x] API: `PUT` y `DELETE /api/recipes/{id}` con autorización y tests (200/404/400, `401/403`).
- [x] UI: página de recetas (lista, búsqueda, detalle, alta, edición, borrado) accesible desde la navegación, con tests de componente.
- [x] Import: `POST /api/recipes/import` con guardas SSRF y parseo Schema.org, con tests (URL interna, HTML sin JSON-LD, fallo de red, idempotencia).
- [x] Rendimiento: RSS del binario `--release --features embed-ui` medido y documentado; presupuestos web y táctiles verificados en `docs/performance.md`.
- [x] `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde (verificado: los tres en verde).
- [x] Sin regresiones en el smoke E2E (37/37 + casos nuevos del recetario) — verificado: smoke de Fase 5 37/37 en esta rama; los smokes del recetario (23/23) e importación (18/18) se verificaron en sus PRs y este change no toca código de producto.

## 20. Fase 7 — Entrega (Docker/release) y producto (recetario/menú)

> Ejecutable ahora. Origen: issues #52–#56. La fase mezcla **entrega** (E1–E2:
> Docker y releases) y **producto** (P1–P3: planificación en la UI, etiquetas y
> volcado de receta a la compra). Cinco changes independientes, una rama/PR por
> issue desde `development`. Metodología SDD + TDD estricto (RED → GREEN →
> REFACTOR), con auditoría por bloque.

### 20.1 Objetivo

Cerrar la promesa de OxiMeal como **self-hosted de binario único desplegable**:
hoy `just build-full` produce un ejecutable con la UI embebida, pero **no hay
`Dockerfile` ni pipeline de release** —el despliegue depende de reproducir el
toolchain a mano— y `deploy/compose.yaml` solo levanta Pocket ID. En paralelo,
completar el **bucle del recetario/menú dentro de la propia app**: el menú
semanal es de **solo lectura** y el estado vacío remite a Valet, no hay etiquetas
libres para clasificar el recetario y no se pueden volcar los ingredientes de una
receta concreta a la lista. Los changes E1–E2 materializan la entrega y los
P1–P3 cierran el producto.

### 20.2 Decisiones de diseño (ADR-32…36)

| # | Decisión | Justificación |
|---|----------|---------------|
| ADR-32 | Imagen **Docker multi-stage** (frontend Node/pnpm → builder Rust → runtime `debian:bookworm-slim` con solo `ca-certificates`/`curl`), usuario **no root**, `VOLUME /app/data`, `HEALTHCHECK` contra `/api/health`, y compilación del binario con `--features embed-ui` | Reproduce el toolchain en un build declarativo y mínimo; el runtime slim mantiene la imagen pequeña y el healthcheck permite a compose esperar al arranque; `embed-ui` conserva el modelo de binario único |
| ADR-33 | **Release por tag `v*`**: puertas de calidad reutilizadas de `ci.yml`, artefactos `x86_64`/`aarch64` (`--release --features embed-ui`, `.tar.gz`+`.sha256`), imagen en GHCR (`semver`+`latest`), notas con **`git-cliff`** desde Conventional Commits + gitmoji, y versionado verificado (tag == `Cargo.toml`/`package.json`), *bump* manual en PR de release | Automatiza la entrega hoy hecha a mano y garantiza coherencia tag↔versión↔imagen↔notas; reutilizar las puertas evita publicar código que no pasa CI; `git-cliff` no añade un servicio externo (se descarta `svu`/`vampus`) |
| ADR-34 | **Planificación nativa en la UI que convive con Valet**; `PATCH /api/meal-plan/{id}` para editar **solo raciones** y endurecer la validación de `POST`; la planificación **requiere conexión** (no se encola en IndexedDB) | Permite crear/quitar/corregir entradas sin salir de la app sin retirar el asistente; el `PATCH` mínimo (sin reemplazo de receta/fecha/tipo/rol) evita conflictos; encolar prometería una persistencia que la validación server-side del `recipe_id` y la creación de `id` no permiten |
| ADR-35 | **Etiquetas libres many-to-many** (`tags` + `recipe_tags`, `slug` normalizado y único, FKs `ON DELETE CASCADE`, migración `0007` aditiva), **fuera** del índice FTS; filtro `GET /api/recipes?tag=<id\|slug>` combinable con `q`/`category`/paginación | Las categorías y `diet_type` ya son ejes cerrados; un vocabulario abierto no obliga a decidir dimensiones de antemano; indexar etiquetas en FTS contaminaría las coincidencias por subcadena y tocaría los triggers `ai/ad/au` |
| ADR-36 | **Volcado de receta a la lista** con `POST /api/shopping-list/from-recipe` (permiso `write:shopping`), **reconsolidación declarativa** de las fuentes de receta registradas (idempotente) y **escalado por raciones** (`solicitadas / receta`) antes de consolidar, **sin** modificar líneas del plan ni manuales ni sumar unidades no aditivas | Reutiliza el motor determinista de `ingredient-consolidation` (canonicalización, normalización, pasillos); reconstruir es idempotente y reescala al repetir, sin duplicar |

### 20.3 Change E1 — `container-image` (issue #52)

- **`Dockerfile` multi-stage** en la raíz (contexto = monorepo): stage
  `frontend` (`pnpm install --frozen-lockfile` + `pnpm build`), stage `builder`
  (imagen oficial `rust`, recibe `frontend/dist` y compila `oximeal-backend`
  `--release --features embed-ui`) y stage `runtime` (`debian:bookworm-slim`,
  usuario no root, `WORKDIR /app`, `EXPOSE 8080`, `VOLUME /app/data`, `ENV` por
  defecto, `HEALTHCHECK` contra `/api/health`, `ENTRYPOINT`).
- **`.dockerignore`** y caché de capas (manifiestos antes que el código).
- **`deploy/compose.yaml`**: nuevo servicio `oximeal` (imagen local o GHCR,
  `env_file`, volumen con nombre para SQLite, red y healthcheck) **manteniendo**
  `pocket-id`; arranque conjunto y modo abierto documentados.
- **Docs:** sección de despliegue en `README.md` (con refresco del estado a
  Fase 6/7), `deploy/README.md` y `deploy/.env.example` actualizado.
- **Specs:** capacidad nueva `deployment` (build reproducible, runtime no root,
  persistencia, healthcheck, compose e instrucciones).
- **Estado:** ✅ implementado y verificado (rama `feature/container-image`; `deploy/verify-image.sh` con 20/20 checks).

### 20.4 Change E2 — `release-automation` (issue #53)

> Estado: ✅ cerrado (PR #59, squash `e729455` a `development`). Implementa el ADR-33.

- **`.github/workflows/release.yml`** (nuevo): disparo `push: tags: ['v*']` +
  `workflow_dispatch` (input `tag`); **nunca** en `pull_request`.
- **Puertas de calidad**: job `quality` que reutiliza `ci.yml` vía
  `workflow_call` (se añade `workflow_call:` a `on:` de `ci.yml`, sin alterar su
  comportamiento en `push`/`pull_request`).
- **Coherencia de versionado**: job `version` ejecuta
  `.github/scripts/check-version.sh <tag>` y falla si el tag no es `v<semver>` o
  no coincide con `[workspace.package] version`, `frontend/package.json` y la
  última versión de `CHANGELOG.md`.
- **Artefactos binarios**: matriz `x86_64-unknown-linux-gnu` (`ubuntu-latest`) y
  `aarch64-unknown-linux-gnu` (`ubuntu-24.04-arm`, runner nativo ARM — repo
  público); se construye el frontend (`pnpm build`) y luego
  `cargo build --release -p oximeal-backend --features embed-ui --target …`; se
  empaqueta `oximeal-<version>-<target>.tar.gz` + `.sha256`.
- **Imagen GHCR**: `ghcr.io/<owner>/oximeal:<version>` y `:latest` mediante
  `docker/build-push-action` (login con `GITHUB_TOKEN`); `latest` solo se publica cuando el tag es el
  más reciente y no es *pre-release*. Plataforma `linux/amd64`.
- **GitHub Release**: job `release` descarga **solo** los artefactos binarios
  (`pattern: oximeal-*`, para no tropezar con el caché de Docker Buildx), genera
  las notas desde la sección curada del `CHANGELOG.md`
  (`.github/scripts/release-notes.sh`, con `git-cliff` de *fallback*) y crea/edita
  la release con `gh release` (`--prerelease` si procede); adjunta
  provenance con `actions/attest-build-provenance` (`id-token: write`,
  `attestations: write`).
- **Notas (`cliff.toml`)**: agrupación por tipo Conventional Commits **con
  gitmoji** (regex `^[^a-zA-Z]*<tipo>`, necesaria porque el *variation selector*
  del emoji cuenta como `\w`), grupos en español coherentes con `CHANGELOG.md` y
  enlaces a issues.
- **Documentación**: sección «Releases / Publicación» en `README.md`.
- **Decisiones (ADR-33)**: runner **ARM nativo** (no `cross`); imagen de release
  **linux/amd64**; `bump` de versión **manual** en PR de release a `main`; tag
  `v<version>` dispara el pipeline. El e2e real es la release **v0.5.0** al
  cerrar P3.
- **Verificación**: `actionlint` sobre los workflows, `git-cliff` local,
  RED→GREEN del script de coherencia, `just ci`/`just ci-ui` y OpenSpec
  `--all --strict`.
- **Specs:** capacidad nueva `release`.

### 20.5 Change P1 — `meal-planning-ui` (issue #54)

- **Backend:** nuevo `PATCH /api/meal-plan/{id}` (permiso `write:plan`) para
  editar **solo `servings`** (`200` con la vista actualizada, `400` si
  `servings < 1`, `404` si no existe); endurecer y testear la validación de
  `POST /api/meal-plan` (`meal_type`, `role`, `date` ISO, `recipe_id` existente,
  `servings >= 1`, 401/403).
- **UI:** alta por slot en `MenuPage`/`DayCard` con selector de receta con
  búsqueda, tipo de comida, rol y raciones; baja con **confirmación**; edición de
  raciones; Valet sigue disponible (convivencia).
- **Offline:** la planificación **requiere conexión** y la UI deshabilita los
  controles y muestra aviso degradado (no se encola).
- **Accesibilidad:** objetivos táctiles ≥48px, gestión de foco, roles `aria`,
  feedback de carga/error.
- **Specs:** deltas en `meal-plan`, `weekly-menu`, `ui-shell` y `offline-writes`.
- **Estado:** ✅ implementado y auditado (rama `feature/meal-planning-ui`, PR pendiente). Incluye el refresco del plan al recuperar la conexión (scenario «Recuperación de la conexión»).

### 20.6 Change P2 — `recipe-tags` (issue #55)

- **Migración `0007`** aditiva: `tags(id, name, slug UNIQUE, created_at)` y
  `recipe_tags(recipe_id, tag_id)` con PK compuesta, FKs `ON DELETE CASCADE` e
  índice `idx_recipe_tags_tag_id`; **no** toca `recipes` ni `recipes_fts`.
- **API:** `GET /api/tags` (`read:recipes`, con conteo), `POST /api/tags`
  (idempotente, `write:recipes`) y `DELETE /api/tags/{id}` (en cascada, sin
  borrar recetas); `POST`/`PUT /api/recipes` aceptan `tags` (reemplazan el
  conjunto) y `GET /api/recipes?tag=` filtra combinado con `q`/`category`.
- **UI:** selector con creación al vuelo en `RecipeForm`, etiquetas en
  `RecipeCard`/detalle y filtro por etiqueta reflejado en la URL.
- **Specs:** capacidad nueva `recipe-tags` + deltas en `recipes`,
  `recipe-catalog` y `recipe-search`.
- **Estado:** ✅ cerrado (PR #61, `a98dbe0`); issue #55 cerrado.

### 20.7 Change P3 — `shopping-from-recipe` (issue #56)

- **Backend:** `POST /api/shopping-list/from-recipe` (permiso `write:shopping`)
  con `{ recipe_id, servings? }` (por defecto las raciones de la receta);
  responde `200` con los grupos resultantes, `400` si `servings` es inválido y
  `404` si la receta no existe.
- **Semántica:** registro en `shopping_recipe_sources` + columna
  `source_recipe_id`; **reconsolidación** idempotente de las recetas registradas
  (repetir no duplica, reescala sin tocar líneas del plan ni manuales).
- **Escalado:** factor `solicitadas/receta` a todas las cantidades antes de
  consolidar; `al gusto` conserva cantidad 0 y no se suman familias
  incompatibles.
- **UI:** acción «Añadir a la lista» en `RecipeDetailPage` con selector de
  raciones, feedback y enlace a `/shopping`; sin conexión se deshabilita.
- **Specs:** deltas en `shopping-list`, `ingredient-consolidation`,
  `recipe-catalog` y `offline-writes`.
- **Estado:** ✅ cerrado (PR #62, `6b23b2b`); issue #56 cerrado.

### 20.8 Orden de ejecución y ramas

1. `feature/container-image` (issue #52) desde `development`.
2. `feature/release-automation` (issue #53), sobre el anterior ya mergeado.
3. `feature/meal-planning-ui` (issue #54).
4. `feature/recipe-tags` (issue #55).
5. `feature/shopping-from-recipe` (issue #56), el último (volcado sobre el
   recetario y la compra ya completos).

Nota: la **entrega va primero** (E1→E2) para poder publicar la **v0.5.0** con el
recetario completo una vez cerrados los P1–P3. Una rama/PR por change desde
`development`.

### 20.9 Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| Build de la imagen y enlazado `glibc`/`rustls` (y healthcheck que necesita `curl`/shell) | Runtime `debian:bookworm-slim` coherente con el builder Rust; se instala `curl` explícitamente y el `HEALTHCHECK` apunta a `/api/health`; si se planteara distroless, el healthcheck pasaría a un probe del orquestador |
| Cross-compile a `aarch64` frágil | Matriz de build con runner nativo ARM o `cross` (se decide y justifica); artefactos verificados con su `.sha256` |
| Convivencia UI↔Valet y validación de planificación | `PATCH` mínimo (solo raciones) y validación server-side endurecida; la UI deshabilita los controles sin conexión en vez de encolar |
| Escalado de recetas con unidades no aditivas | Reutilizar el motor de `ingredient-consolidation`; `al gusto` conserva cantidad 0 y no se suman familias incompatibles; tests de escalado |
| Migración de etiquetas que contamine el FTS o reconstruya tablas | Migración **aditiva** (`0007`) que no toca `recipes` ni `recipes_fts`; etiquetas fuera del FTS y filtro por igualdad con `idx_recipe_tags_tag_id` |
| Volumen SQLite con permisos no root | `VOLUME /app/data` con usuario no root y `DATABASE_URL` por defecto apuntando al volumen; permisos verificados al arrancar el contenedor |

### 20.10 Criterios de aceptación (DoD de la Fase 7)

- [x] La imagen Docker construye y arranca con healthcheck OK y datos persistentes en el volumen.
- [x] Un tag `v*` publica binarios (x86_64/aarch64) + imagen GHCR + notas generadas, y **no** publica en PR.
- [x] Planificación desde la UI con tests (alta en slot, baja con confirmación, edición de raciones, validación y accesibilidad).
- [x] Etiquetas con migración + API (CRUD/filtro `tag`) y tests, fuera del FTS.
- [x] Volcado de receta con escalado por raciones, fusión/reconsolidación sin duplicados y respeto de no aditivos, con tests.
- [x] `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde en las cinco ramas.
- [x] Sin regresiones en el smoke E2E.
- [x] Los cinco changes archivados (P3 `2026-11-04-shopping-from-recipe`, notas de release `2026-11-04-release-notes-from-changelog`) y la Fase 7 marcada como cerrada en este plan.
- [x] La release **v0.5.0** publicada end-to-end: tag anotado `v0.5.0`, `release.yml` 8/8 jobs en verde, GitHub Release con notas del CHANGELOG, `.tar.gz` + `.sha256` (checksum validado) e imagen `ghcr.io/atareao/oximeal:0.5.0` y `:latest`.

### 20.11 Cierre de la Fase 7 — release v0.5.0

Cierre real de la fase: la release **v0.5.0** quedó publicada end-to-end.

- **Bump a 0.5.0** (manual, ADR-33) en `[workspace.package] version` de
  `Cargo.toml`, `backend/Cargo.toml` (versión explícita), `frontend/package.json`
  y `Cargo.lock`; sección `## [0.5.0] - 2026-11-04` en `CHANGELOG.md`. PR de
  release #63 → `main` (`f332e04`); tag anotado `v0.5.0`.
- **Workflow `release.yml`**: `quality` (reutiliza `ci.yml`) → `version` →
  `artifacts` (x86_64 + aarch64) / `image` (GHCR) → `release`. Run final
  `41785535065`: **8/8 jobs en verde**, incluido `GitHub Release`.
- **Publicación verificada**: GitHub Release `v0.5.0` marcada como *Latest*, con
  notas de la sección curada del `CHANGELOG.md`, assets
  `oximeal-0.5.0-{x86_64,aarch64}-unknown-linux-gnu.tar.gz` + `.sha256`
  (checksum recomprobado) y atestación de provenance; imagen
  `ghcr.io/atareao/oximeal:0.5.0` y `:latest`.
- **Hotfixes de CI necesarios** (job `release` falló al publicar; se resolvieron
  por PR a `main`):
  1. `#64` (`4f8d275`): descargar solo los artefactos binarios
     (`pattern: oximeal-*`), excluyendo el caché de Docker Buildx que rompía la
     descarga.
  2. `#65` (`a4dc2bb`): `git-cliff-action@v4` no admite la entrada `output`;
     escribir `RELEASE_NOTES.md` con `-o` en `args`.
  3. `#66` (`d31b060`): **causa estructural** — `main` integra por *squash-merge*,
     así que `git-cliff --current` solo ve un commit; las notas pasan a tomarse
     del `CHANGELOG.md` (script `.github/scripts/release-notes.sh`, `git-cliff`
     de *fallback*). Registrado como change OpenSpec
     `2026-11-04-release-notes-from-changelog` (requisito `release` MODIFIED).
- **Sincronización**: `main` (`f332e04` + hotfixes) se integra en `development`.

**DoD del cierre:** los cinco changes de la Fase 7 archivados, `main` con la
release publicada y `development` sincronizado; `openspec validate --all
--strict` en verde.

### 20.12 Mantenimiento post-release y auditoría de seguridad — hotfix #80

Cierre del ciclo de mantenimiento tras la v0.5.0, con el backlog de seguridad
atendido por SDD + TDD.

- **A — Validación del pipeline**: `workflow_dispatch` con `tag=v0.5.0`
  (run `41790282569`, **8/8 jobs**) validó de paso los *bumps* de acciones. El
  job `release` fallaba con `exit 127` al revalidar tags anteriores al
  generador de notas; ahora hace checkout del ref que dispara la ejecución
  (`ref: ${{ github.sha }}`) mientras `artifacts`/`image` siguen compilando
  desde el tag. Change `release-notes-generator-from-ref`, PR #70.
- **B — Mantenimiento de CI**: acciones a mayores con Node 24 (`checkout@v7`,
  `setup-node@v7`, `upload-artifact@v7`, `download-artifact@v8`,
  `pnpm/action-setup@v6`, `attest-build-provenance@v4`, `docker/*`), runners
  fijados a `ubuntu-24.04`, `check-version.sh` valida también
  `backend/Cargo.toml` y nueva puerta **Scripts (shell)** (`shellcheck` +
  `.github/scripts/test-gh-scripts.sh`, 14 casos). Change `ci-maintenance`,
  PR #69.
- **C — Dependabot**: `.github/dependabot.yml` para `cargo`, `npm`/pnpm,
  `github-actions` y `docker`, con PRs a `development` (gitflow), cadencia
  semanal y grupos `minor`+`patch`. Nueva capability `dependencies`. PR #71.
  Primera ejecución: 7 PRs (#73–#79) todas con base `development`.
- **Sincronización** `main` → `development` (PR #72).
- **E — Auditoría de seguridad run-1** (import SSRF/ADR-30 + BFF OIDC): **1
  hallazgo MEDIUM** (#80) y **12 notas de endurecimiento** (#81); el SSRF del
  import queda limpio (esquema, canonicalización de IPv4/IPv6, RFC1918/ULA/
  link-local/CGNAT, redirecciones revalidadas, *pin* DNS anti-TOCTOU,
  `Content-Type` y tope de tamaño). Artefactos en la auditoría local; los
  hallazgos se reportan como issues, no como notas.
- **Hotfix #80 (CWE-601, open redirect post-login)**: `sanitize_return_to`
  solo vetaba `/` inicial, `//` y `\`; un `return_to=/%09/evil.com` pasaba y
  el `302 Location: /<TAB>/evil.com` se normalizaba en el navegador a
  `//evil.com`. Ahora se rechaza cualquier `char::is_control` y el destino cae
  a `/`. Ciclo completo: change aprobado y archivado, **RED reproducido
  end-to-end** (sin el fix, el callback emitía `Location: "/\t/evil.com"`),
  unitarios + integración, `just ci`/`just ci-ui`/`openspec validate --all
  --strict` en verde, PR #82 → `main` (`465c7f6`) y sync PR #83 →
  `development` (`cebf024`).
- **Estado**: `main` y `development` con árbol idéntico (0 ficheros), #80
  cerrado, pendiente solo el backlog de endurecimiento #81 y la reconciliación
  de las PRs de Dependabot.

### 20.13 — Endurecimiento del BFF de sesión y cliente OIDC (#81)

Primer change del backlog de endurecimiento, aprobado por el usuario con
alcance **BFF de sesión/OIDC** y las notas (3) y (12e) como «documentar, sin
código».

- **Change `security-hardening-auth`** (SDD): 5 requisitos tocados en
  `auth-session` (MODIFIED callback + ADDED vida absoluta), `api-authorization`
  (ADDED `no-store`), `oidc-config` (MODIFIED entorno + ADDED secretos fuera de
  logs) y `deployment` (MODIFIED entorno). 18 escenarios. Archivado como
  `2026-11-05-security-hardening-auth`.
- **TDD**: RED con 8 tests unitarios y 5 de integración fallando por la causa
  esperada (22 tests de auth previos en verde); GREEN mínimo; `clippy -D
  warnings` limpio; suite con y sin `embed-ui` en verde.
- **Implementación**: `Debug` manual con `[REDACTED]`; cookie por defecto
  `__Host-oximeal_session` con validación del prefijo; fail-closed de
  `SESSION_COOKIE_SECURE` con escape `ALLOW_INSECURE_COOKIE`; vida absoluta
  `SESSION_ABSOLUTE_TTL_SECS` (12 h) comprobada antes de `ensure_fresh` en
  middleware y `/auth/me`; `nbf`/`azp` en `verify_id_token`; capa
  `Cache-Control: no-store` para peticiones con cookie o `Bearer`.
- **Docs**: `deploy/README.md` (prefijo, TTL absoluto, escape y HSTS/
  `Referrer-Policy` en el proxy), `.env.example`, `deploy/.env.example`,
  `deploy/oidc-dev.md` y `README.md`.
- **Puertas**: `just ci`, `just ci-ui`, `openspec validate --all --strict`
  (27 specs) y CI del PR (4/4) en verde. PR #87 → `development` (`68cde0c`).
- **Pendiente del backlog #81**: cifrado en reposo (11), RP-initiated logout
  (12d) y SSRF del import (1, 2, 4, 5, 6, 7).
