# PLAN.md — OxiMeal

> Plan maestro de implementación. Generado por el Project Director.
> Spec de origen: «Especificación Técnica y Funcional: OxiMeal v1.0».
> **Alcance ejecutable en esta ronda: Fase 1 (Core Backend + SQLite).**

## 1. Visión y alcance

OxiMeal es un gestor self-hosted de menús semanales, recetas y listas de la compra, mobile-first, offline-capable y desplegable como **binario único** (Axum + `rust-embed`), con un orquestador LLM interno (OpenRouter) conectado a un **servidor MCP** que ejecuta acciones reales sobre SQLite.

**En esta ronda:** construir y dejar en verde el núcleo backend (Fase 1).
**Fuera de alcance ahora:** UI React (F2), MCP+Chat (F3), PWA/Offline (F4).

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
- **Fase 2 — Frontend:** React 19 + Antd v6; layout responsive (Sider desktop / bottom-nav mobile); calendario semanal; lista de la compra mobile-first por pasillos (objetivos ≥48px, strike-through + reordenado + haptics); `rust-embed` para binario único.
- **Fase 3 — Integración con Valet (en el repo Valet, NO en OxiMeal):** cliente `src/services/oximeal.rs` + tools `src/tools/oximeal.rs` (con permisos) que consumen la Agent API. **No** se construye orquestador/MCP/chat en OxiMeal (Valet descartó MCP; usa integración nativa por servicio). La pestaña 🤖 del PWA enlaza/embebe el chat de Valet.
- **Fase 4 — PWA/Offline:** `manifest.json` + iconos adaptativos; Service Worker (Stale-While-Revalidate para activos, Network-First con fallback IndexedDB para la lista); pruebas táctiles y de rendimiento.

## 7. Estrategia de testing

- **Unit (dominio):** `#[cfg(test)]` en `domain/*` — canonicalización de unidades, normalización de nombres, categorización, consolidación (sumas, no-aditivos, conflictos de unidad).
- **Integración (API):** `backend/tests/*.rs` levantan el `Router` de `lib.rs` con SQLite temporal migrada (`tempfile`) y usan `oneshot`/`axum-test`.
- **Contrato FTS:** caso que inserta recetas y verifica coincidencia por substring (trigram) y fallback LIKE.
- **Puertas obligatorias:** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `openspec validate phase-1-core-backend --strict`.

## 8. Git flow (resumen; detalle en GIT_FLOW.md)

- Ramas: `main` (producción) · `development` (integración) · `feature/*` desde `development` · `hotfix/*` desde `main`.
- Nunca commits directos a `main`/`development`: todo por PR.
- Commits: `<gitmoji> <tipo>(<scope>): <mensaje>` (Conventional Commits + gitmoji).

## 9. Criterios de aceptación (Definition of Done, Fase 1)

- [x] Workspace compila: `cargo build` OK.
- [x] Migraciones aplican sobre base nueva; FTS5+trigram verificados en runtime.
- [x] Motor de consolidación con cobertura de tests (unidades, acentos, no-aditivos, conflictos).
- [x] Los 11 endpoints de §6 implementados y cubiertos por tests de integración.
- [x] `cargo test` en verde; `clippy -D warnings` limpio; `fmt --check` limpio.
- [x] `openspec validate phase-1-core-backend --strict` en verde.
- [x] `/api/health` responde 200; Swagger accesible en `/api/docs`.
- [x] Queries sqlx parametrizadas en runtime (sin dependencia de .sqlx).
- [x] Agent API `/api/agent` implementada y cubierta por tests de integración.

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| FTS5/trigram no disponible en el build de `libsqlite3-sys` | Task T3 verifica en runtime; fallback a `LIKE` para `q.len()<3`. |
| Query trigram insensible a acentos limitada | `normalize.rs` aplica folding de acentos antes de indexar/buscar. |
| Edición 2024 requiere toolchain reciente | `rust-toolchain.toml` fija stable ≥1.97 (presente en el entorno). |
| Ambigüedad Schema.org en import | Fase 1 acepta solo JSON validado; el parser Schema.org se completa en F2/F3. |
| Consumo de RAM <20 MB (objetivo global) | Fase 1 no añade runtime LLM; se mide con el binario monolítico en F2. |

## 11. Trazabilidad con la especificación

- §3 Modelo de datos → T3, T7.
- §4 Tools MCP → Agent API HTTP (`/api/agent`), consumida por Valet.
- §5 UI mobile-first → Fase 2.
- §6 Endpoints REST → T8–T11.
- §7 Roadmap → secciones 5 y 6 de este plan.
