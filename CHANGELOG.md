# Changelog

Todas las novedades relevantes de **OxiMeal** se documentan en este fichero.

OxiMeal es un gestor **self-hosted** de menús semanales, recetas y listas de la
compra: una API *headless* en Rust (Axum + SQLite) con el frontend React
embebido en el propio binario.

El formato de este fichero sigue
[Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y este proyecto se
adhiere al [Versionado Semántico](https://semver.org/lang/es/).

## [No publicado]

### Añadido

### Cambiado

### Obsoleto

### Eliminado

### Corregido

### Seguridad

## [0.2.0] - 2026-11-02

Primera release etiquetada. Integra la Fase 1 (núcleo backend) y la Fase 2
(frontend + servido embebido), que entraron en `development` mediante los PRs
[#1](https://github.com/atareao/oximeal/pull/1) y
[#7](https://github.com/atareao/oximeal/pull/7). La deuda diferida detectada en
las auditorías (`rust-reviewer`, `sqlite-expert` y `react-reviewer`) quedó
registrada como issues #2–#13.

> Nota: no existe un tag anterior. Al ser la primera release etiquetada, el
> enlace de `[0.2.0]` compara con el commit inicial del repositorio.

### Añadido

#### Núcleo backend

- Workspace Cargo (edition 2024), `rust-toolchain.toml` y configuración base del
  repositorio.
- Persistencia SQLite con WAL, claves foráneas y `busy_timeout`.
- Migraciones versionadas `0001_init`, `0002_fts_triggers` y
  `0003_fts_diacritics_and_indexes`: búsqueda FTS5 *external-content* con
  trigramas y plegado de diacríticos.
- Dominio puro (sin IO): canonicalización de unidades, normalización de
  nombres, categorización por pasillo, fechas canónicas y motor de consolidación
  de la lista de la compra.
- Suite de tests del backend (dominio, API, FTS5 y Agent API): 100 tests.

#### API

- API REST con Axum: `/api/recipes`, `/api/meal-plan`, `/api/shopping-list` y
  `/api/health` (12 rutas).
- Documento **OpenAPI 3.1** y **Swagger UI** servidos por el propio backend.
- Validación de peticiones y errores tipados.

#### Agent API

- Cinco endpoints bajo `/api/agent` para el consumo desde Valet:
  `recent-recipes`, `search-recipes`, `weekly-plan`, `shopping-items` y
  `shopping/completed`.

#### Frontend

- React 19 + Vite + TypeScript estricto, Ant Design v6, TanStack Query y
  react-router v7.
- Shell responsive: `Sider` en escritorio y *bottom-nav* en móvil, con objetivos
  táctiles ≥48 px.
- Menú semanal de lunes a domingo con comidas y cenas (COMIDA/CENA).
- Modo compra por pasillos con *haptics*, alta rápida de artículos, limpieza de
  comprados y consolidación.
- *Code-splitting* de la build del frontend.
- Suite de tests del frontend: 73 tests.

#### Servido embebido

- Frontend embebido en el binario con `rust-embed` (feature `embed-ui`) y
  *fallback* SPA para el enrutado del cliente.

#### Tooling

- `justfile` con recetas de desarrollo (backend y UI) y puerta de calidad
  (`fmt-check`, `lint`, `test`, `test-embed-ui`, y comprobación de tipos y build
  de la UI).

### Documentación

- `README.md` con arquitectura, requisitos, desarrollo y configuración.
- `GIT_FLOW.md` con el flujo de ramas y las convenciones del repositorio.
- Especificaciones OpenSpec de las fases 1 y 2 archivadas y consolidadas.
- Guía de la arquitectura *headless/API-first*: el asistente conversacional
  **Valet** es un proyecto hermano que consume la Agent API por HTTP, sin
  orquestador LLM, sin servidor MCP y sin chat dentro de OxiMeal.

[No publicado]: https://github.com/atareao/oximeal/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/atareao/oximeal/compare/04b8f6162fe54101a21ba4749641f4af610c6ca7...v0.2.0
