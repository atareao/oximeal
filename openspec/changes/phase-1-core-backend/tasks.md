# Tasks — Fase 1 (TDD: RED → GREEN → REFACTOR)

## 1. Estructura
- [ ] 1.1 Workspace Cargo raíz y crate `backend`
- [ ] 1.2 Toolchain, .gitignore, .env.example, README

## 2. Datos
- [ ] 2.1 Migración 0001_init (recetas, plan, compra + índices/CHECKs)
- [ ] 2.2 Migración 0002_fts_triggers (FTS5 trigram + triggers)
- [ ] 2.3 Verificación runtime FTS5 + trigram
- [ ] 2.4 Config + pool (WAL, FK, busy_timeout) + migrate!

## 3. Dominio (test-first)
- [ ] 3.1 units: canonicalización y parseo
- [ ] 3.2 normalize: acentos, artículos, singular
- [ ] 3.3 category: heurística pasillo
- [ ] 3.4 consolidate: suma, no-aditivos, conflictos de unidad

## 4. API
- [ ] 4.1 AppError + AppState + router factoría
- [ ] 4.2 GET/POST/GET:id /api/recipes
- [ ] 4.3 GET/POST/DELETE /api/meal-plan
- [ ] 4.4 GET/PATCH /api/shopping-list, POST consolidate, DELETE completed
- [ ] 4.5 GET /api/health, OpenAPI + Swagger

## 5. Cierre
- [ ] 5.1 justfile + cargo sqlx prepare (.sqlx/)
- [ ] 5.2 cargo test / clippy -D warnings / fmt --check en verde
- [ ] 5.3 openspec validate --strict en verde
