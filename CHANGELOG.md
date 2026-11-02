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

- **Recetario:** gestión completa de recetas con `PUT /api/recipes/{id}` (reemplazo total en una transacción `BEGIN IMMEDIATE`; devuelve `409` si la receta está referenciada por una entrada del plan) y `DELETE /api/recipes/{id}` (borrado duro), bajo el permiso `write:recipes`; nueva página de recetas en el frontend (`/recipes` y `/recipes/:id`) con listado, búsqueda con debounce, paginación, alta, edición y borrado con confirmación, accesible desde el Sider y la barra inferior, con degradado offline y objetivos táctiles ≥48px (#45).

### Cambiado

### Obsoleto

### Eliminado

### Corregido

### Seguridad

## [0.4.0] - 2026-11-02

Fase 5: **PWA/Offline** (issues #29, #30, #31; arreglos #35 y #36). Convierte
OxiMeal en una PWA instalable y *offline-first*: shell precacheado con Service
Worker propio, lectura sin conexión desde IndexedDB con modo degradado, y cola
de escrituras con alta idempotente que se sincroniza sola al recuperar la
conexión.

### Añadido

- **Frontend:** linting estático de accesibilidad con `eslint-plugin-jsx-a11y` y formato verificado con Prettier, integrados en `just ci-ui` y en CI (#11).
- **Frontend:** shell PWA instalable con manifiesto, iconos adaptativos y Service Worker propio que precachea el shell y los assets con hash; recarga sin conexión y actualización de caché versionada por build (#29).
- **Frontend:** caché de datos en IndexedDB con hidratación al arrancar y modo degradado: sin conexión o con la sesión caducada se muestran los datos guardados con un aviso, en lugar de vaciar la vista o redirigir al login; el cierre de sesión limpia la caché (#30).
- **Offline:** cola de escrituras en IndexedDB: los cambios de la lista de la compra hechos sin conexión se encolan, se aplican de forma optimista y se sincronizan en orden al recuperar la conexión. El alta es idempotente por `client_id` (migración 0006) para no duplicar ítems, una operación inválida (4xx) se descarta avisando al usuario y el marcado se resuelve por el `updated_at` más reciente (#31).

### Cambiado

- **Frontend:** antd deja de descargarse en el arranque (carga diferida vía `React.lazy` + splitting por grupos); grafo estático inicial 761,3 kB → **293,7 kB** con presupuesto de 500 kB verificado en `just ci-ui` y CI (#13).
- **Frontend:** haptics documentados como Vibration API (`navigator.vibrate`), con no-op en navegadores sin soporte; se descarta `web-haptics` (#10).

### Obsoleto

### Eliminado

### Corregido

- **Modo abierto inservible**: con `OIDC_ENABLED=false`, `GET /auth/me` caía al
  fallback SPA y devolvía `index.html` con 200 en lugar del 404 JSON que el
  frontend espera, lo que dejaba la aplicación embebida bloqueada en
  `/auth/login`. Las rutas `/auth/*` se clasifican ahora como rutas de API en el
  fallback ([#36](https://github.com/atareao/oximeal/issues/36)).
- **La cola de escrituras offline no se drenaba al recuperar la conexión**: si el
  drenado fallaba por red mientras `navigator.onLine` seguía en `true`, nadie
  reintentaba y el aviso de cambios pendientes quedaba indefinidamente. Un nuevo
  planificador (`drainScheduler`) reintenta con backoff exponencial acotado
  (2→30 s), sin bucle ante 401/403, y reintenta al volver al primer plano o
  recuperar el foco ([#35](https://github.com/atareao/oximeal/issues/35)).

### Seguridad

## [0.3.0] - 2026-11-02

Fase 4: **autenticación OIDC con Pocket ID** (PR
[#15](https://github.com/atareao/oximeal/pull/15)). Cierra la instancia
self-hosted mediante un BFF con cliente confidencial, sesión server-side y
autorización por permisos. Retira el API key estático del issue #5.

### Añadido

#### Autenticación OIDC (backend)

- BFF `/auth/{login,callback,me,logout}`: *authorization code* + PKCE `S256`,
  `state`/`nonce`, intercambio de código en servidor, **sesión server-side** en
  SQLite y cookie `httpOnly` + `Secure` + `SameSite=Lax`. El navegador nunca ve
  tokens.
- Descubrimiento OIDC y **JWKS** con caché y refresco por `kid`; validación de
  ID tokens y de access tokens **RFC 9068** (`iss`/`aud`/`exp`/`nbf`,
  `typ=at+jwt` y allowlist de algoritmos sin `none`).
- **Autorización por permisos** del modelo *APIs and permissions* en doble vía
  (cookie de sesión o `Authorization: Bearer`), con 401 (y
  `WWW-Authenticate: Bearer`) / 403.
- **CSRF** (`X-CSRF-Token`) en las mutaciones por cookie, renovación transparente
  con *refresh token* y puerta de entrada por grupos permitidos
  (`OIDC_ALLOWED_GROUPS`).
- Agent API **M2M** con permisos dedicados `agent:read`, `agent:plan` y
  `agent:shopping` (contrato sin cambios).
- Migración `0004_sessions` para las sesiones server-side.

#### Frontend

- Contexto de sesión (`/auth/me`), guard de rutas con retorno al destino, manejo
  de 401 (re-login) y 403 (aviso sin destruir la sesión), cabecera CSRF en las
  mutaciones, e identidad y cierre de sesión en la interfaz.
- Suite de tests del frontend: 110 tests.

#### Despliegue y documentación

- `deploy/compose.yaml` (Pocket ID v2.19.1) y `deploy/oidc-dev.md`: API/resource,
  permisos, cliente user-delegated y cliente M2M.
- README y `.env.example` con las variables OIDC/`SESSION_*`, la tabla de
  permisos por endpoint y la migración del API key.
- Especificaciones OpenSpec de la Fase 4 archivadas y consolidadas
  (`agent-api-auth`, `api-authorization`, `auth-session`, `auth-ui`,
  `oidc-config`).

### Cambiado

- El CORS deja de ser `Any` y pasa a la lista configurable `ALLOWED_ORIGINS`.
- Todas las rutas, salvo `GET /api/health`, el documento OpenAPI y la Swagger UI,
  pasan a exigir credenciales.

### Obsoleto

- `OXIMEAL_API_KEY`: el API key estático queda obsoleto y ya no se acepta; se
  sustituye por **M2M OIDC** (issue #5).

### Seguridad

- La instancia puede cerrarse: sesión server-side, tokens nunca expuestos al
  navegador, CSRF activo y autorización por permisos mínimos.

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

[No publicado]: https://github.com/atareao/oximeal/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/atareao/oximeal/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/atareao/oximeal/compare/04b8f6162fe54101a21ba4749641f4af610c6ca7...v0.2.0
