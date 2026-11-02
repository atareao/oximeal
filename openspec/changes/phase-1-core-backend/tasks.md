# Tasks — Fase 1 (TDD: RED → GREEN → REFACTOR)

## 1. Estructura
- [x] 1.1 Workspace Cargo raíz y crate `backend`
- [x] 1.2 Toolchain, .gitignore, .env.example, README

## 2. Datos
- [x] 2.1 Migración 0001_init (recetas, plan, compra + índices/CHECKs)
- [x] 2.2 Migración 0002_fts_triggers (FTS5 trigram + triggers)
- [x] 2.3 Verificación runtime FTS5 + trigram
- [x] 2.4 Config + pool (WAL, FK, busy_timeout) + migrate!

## 3. Dominio (test-first)
- [x] 3.1 units: canonicalización y parseo
- [x] 3.2 normalize: acentos, artículos, singular
- [x] 3.3 category: heurística pasillo
- [x] 3.4 consolidate: suma, no-aditivos, conflictos de unidad

## 4. API
- [x] 4.1 AppError + AppState + router factoría
- [x] 4.2 GET/POST/GET:id /api/recipes
- [x] 4.3 GET/POST/DELETE /api/meal-plan
- [x] 4.4 GET/PATCH /api/shopping-list, POST consolidate, DELETE completed
- [x] 4.5 GET /api/health, OpenAPI + Swagger

## 5. Cierre
- [x] 5.1 justfile + cargo sqlx prepare (.sqlx/)
- [x] 5.2 cargo test / clippy -D warnings / fmt --check en verde
- [x] 5.3 openspec validate --strict en verde
