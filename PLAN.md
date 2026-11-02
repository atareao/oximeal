# PLAN.md — OxiMeal

> Plan maestro de implementación. Generado por el Project Director.
> Spec de origen: «Especificación Técnica y Funcional: OxiMeal v1.0».
> **Alcance ejecutable en esta ronda: Fase 1 (Core Backend + SQLite).**

## 1. Visión y alcance

OxiMeal es un gestor self-hosted de menús semanales, recetas y listas de la compra, mobile-first, offline-capable y desplegable como **binario único** (Axum + `rust-embed`), con un orquestador LLM interno (OpenRouter) conectado a un **servidor MCP** que ejecuta acciones reales sobre SQLite.

**En esta ronda:** construir y dejar en verde el núcleo backend (Fase 1).
**Fuera de alcance ahora:** UI React (F2), MCP+Chat (F3), PWA/Offline (F5).

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
- **Fase 5 — PWA/Offline:** `manifest.json` + iconos adaptativos; Service Worker (Stale-While-Revalidate para activos, Network-First con fallback IndexedDB para la lista); pruebas táctiles y de rendimiento. Debe tolerar la sesión OIDC caducada sirviendo datos cacheados mientras no haya autenticación.

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
| Ambigüedad Schema.org en import | Fase 1 acepta solo JSON validado; el parser Schema.org se completa en F2/F3. |
| Consumo de RAM <20 MB (objetivo global) | Fase 1 no añade runtime LLM; la medición real se hará sobre el binario monolítico (feature `embed-ui`), **pendiente** de ejecutar; no se fijan cifras aún. |
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

- [ ] Aplicación instalable en Android (Chrome) e iOS (Safari) con iconos correctos.
- [ ] Shell arranca sin conexión tras una visita previa.
- [ ] Lista de la compra visible sin conexión y con sesión caducada, con aviso de modo degradado.
- [ ] Mutaciones offline encoladas y sincronizadas sin duplicados al recuperar la conexión.
- [ ] `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde en las tres ramas.
- [ ] Los tres changes archivados y la Fase 5 marcada como cerrada en este plan.
