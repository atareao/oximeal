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

- **Seguridad (sesión):** vida absoluta de la sesión (`SESSION_ABSOLUTE_TTL_SECS`, 12 h por defecto) medida desde el login. El refresco de tokens ya no puede extenderla indefinidamente: superado el plazo, las rutas protegidas responden `401` y `/auth/me` devuelve `authenticated: false` (#81).
- **Seguridad (API):** las respuestas a peticiones autenticadas (cookie de sesión o `Bearer`) llevan `Cache-Control: no-store` para evitar que datos privados queden en cachés intermedias (#81).

### Cambiado

- **Seguridad (cookie):** el nombre por defecto de la cookie de sesión pasa a `__Host-oximeal_session`. El prefijo `__Host-` obliga a `Secure`, `Path=/` y sin `Domain`, y el arranque falla si se combina con `SESSION_COOKIE_SECURE=false`. **Impacto de despliegue:** al actualizar se invalidan las sesiones existentes y cada usuario vuelve a iniciar sesión una vez (#81).
- **Seguridad (configuración):** con `OIDC_ENABLED=true`, `SESSION_COOKIE_SECURE=false` ya no solo avisa: impide arrancar salvo que se defina `ALLOW_INSECURE_COOKIE=true` (escape explícito para desarrollo en localhost) (#81).
- **Seguridad (OIDC):** la validación del ID token rechaza `nbf` en el futuro y un `azp` distinto del `client_id` (#81).
- **Seguridad (logs):** `Debug` de `Config`, `OidcConfig` y `SessionConfig` redacta `client_secret` y `SESSION_SECRET` como `[REDACTED]` (#81).
- **Despliegue:** `deploy/README.md` documenta el prefijo `__Host-`, la vida absoluta, el escape `ALLOW_INSECURE_COOKIE` y que HSTS/`Referrer-Policy` corresponden al reverse-proxy (#81).

## [0.5.1] - 2026-11-04

### Añadido

- **Mantenimiento:** configuración de **Dependabot** (`.github/dependabot.yml`): actualizaciones semanales de `cargo` (workspace de la raíz), `npm`/pnpm (`frontend/`), `github-actions` y `docker`, con PRs dirigidas a `development` (gitflow), grupos `minor`+`patch` por ecosistema y las `major` en PRs separadas.

### Cambiado

- **Dependencias:** `typescript` 6.0.3 (mayor; tipado, tests y build en verde) y la imagen de build del frontend pasa a `node:26-alpine` (stage `frontend`, verificado con `podman build`).
- **CI:** acciones de los workflows actualizadas a las mayores actuales (Node 24): `actions/checkout@v7`, `setup-node@v7`, `upload-artifact@v7`, `download-artifact@v8`, `pnpm/action-setup@v6`, `attest-build-provenance@v4` y las `docker/*` (`build-push-action@v7`, `login-action@v4`, `metadata-action@v6`, `setup-buildx-action@v4`); runners fijados a `ubuntu-24.04` (incluida la matriz de binarios), sin depender de `ubuntu-latest`.
- **CI:** la coherencia de versionado ahora incluye la `version` explícita de `backend/Cargo.toml` (no hereda de `[workspace.package]`) y se añade la puerta **Scripts (shell)** a `ci.yml` (`shellcheck` + `.github/scripts/test-gh-scripts.sh`, 14 casos).

### Corregido

- **Seguridad (auth):** `GET /auth/login` aceptaba un `return_to` con caracteres de control (TAB/CR/LF). Como el navegador los elimina al parsear la URL, un valor como `/%09/evil.com` se convertía en `//evil.com` y el 302 posterior al login terminaba en un origen externo (*open redirect*, CWE-601), explotable para *phishing* con el dominio legítimo. `sanitize_return_to` ahora rechaza cualquier carácter de control y usa `/` como destino de retorno (#80).
- **CI/Release:** el job `release` tomaba el generador de notas y el `CHANGELOG.md` del árbol del **tag**; al revalidar un tag anterior al generador, el step fallaba con `exit 127` (`release-notes.sh: No such file or directory`). Ahora ese job hace checkout del commit que disparó la ejecución (`github.sha`), mientras los jobs de binarios e imagen siguen construyendo desde el tag.
- **CI/Release:** las notas de la release reproducen también el encabezado `## [X.Y.Z] - fecha` de la sección del `CHANGELOG.md`.
- **CI/Release:** el job `release` de `release.yml` descargaba **todos** los artefactos del run (incluido el caché de Docker Buildx, que rompía la descarga con «Unable to download and extract artifact»); ahora sube/baja los binarios con nombre explícito y `pattern: oximeal-*` (#64).
- **CI/Release:** las notas de la release fallaban en dos frentes —`orhun/git-cliff-action@v4` no admite la entrada `output` y no escribía `RELEASE_NOTES.md`, y con `main` integrado por *squash-merge* `git-cliff --current` solo veía un commit—; ahora se generan desde la sección curada del `CHANGELOG.md` con `.github/scripts/release-notes.sh` (`git-cliff` de *fallback*) (#65, #66).

## [0.5.0] - 2026-11-04

### Añadido

- **Recetario:** gestión completa de recetas con `PUT /api/recipes/{id}` (reemplazo total en una transacción `BEGIN IMMEDIATE`; devuelve `409` si la receta está referenciada por una entrada del plan) y `DELETE /api/recipes/{id}` (borrado duro), bajo el permiso `write:recipes`; nueva página de recetas en el frontend (`/recipes` y `/recipes/:id`) con listado, búsqueda con debounce, paginación, alta, edición y borrado con confirmación, accesible desde el Sider y la barra inferior, con degradado offline y objetivos táctiles ≥48px (#45).

- **Recetario:** importación de recetas desde una URL con `POST /api/recipes/import` (permiso `write:recipes`): descarga server-side con guardas SSRF (solo `http`/`https`, pin de IP frente a DNS rebinding, veto de loopback/RFC1918/link-local, redirecciones acotadas, timeout y tope de tamaño) y parseo del JSON-LD Schema.org `@type: Recipe` (incl. `@graph`), idempotente por `source_url`; diálogo «Importar desde URL» en la página de recetas y allowlist opcional `IMPORT_ALLOWED_HOSTS` para orígenes en LAN (#46).

- **Entrega:** imagen de contenedor **multi-stage** reproducible (`podman build`/`docker build`) con la UI embebida (`--features embed-ui`), runtime mínimo `debian:bookworm-slim` como usuario no root (`oximeal`, UID/GID 10001), `HEALTHCHECK` contra `/api/health`, volumen persistente `/app/data` y `.dockerignore`; orquestación con `deploy/compose.yaml` junto a Pocket ID y guía de despliegue en `deploy/README.md` (#52).

- **Rendimiento:** verificación en dispositivo real con la receta `just perf`, que mide el RSS estable/pico del binario release con la UI embebida (nuevo `[profile.release]`: `lto`, `codegen-units=1`, `panic="abort"` y `strip`) y audita la web por CDP (FCP, transferencia del grafo inicial declarado y objetivos táctiles ≥48px); presupuestos y resultados documentados en `docs/performance.md` — RSS estable ~11 MB, FCP ~40 ms y transferencia ~317 kB. Es una medición on-demand, no un *gate* de CI (ADR-31) (#47).

- **Entrega:** automatización de releases con el workflow `release.yml` disparado por tags `v*` (`workflow_dispatch` opcional): puertas de calidad reutilizadas de `ci.yml` (`workflow_call`), artefactos binarios `x86_64`/`aarch64` (`.tar.gz` + `.sha256`), imagen en GHCR (`<version>` y `latest`) y GitHub Release con notas generadas por `git-cliff` (`cliff.toml`) desde Conventional Commits + gitmoji; verificación de coherencia tag ↔ `Cargo.toml` ↔ `frontend/package.json` ↔ `CHANGELOG.md` (#53).

- **Planificación de comidas:** el menú semanal pasa a ser editable desde la propia interfaz: `PATCH /api/meal-plan/{id}` (permiso `write:plan`) para corregir las raciones sin recrear la entrada (`200` con la vista actualizada, `400` si `servings < 1`, `404` si no existe) y validación endurecida de `POST /api/meal-plan` (receta existente); alta por slot con selector de receta con búsqueda, tipo de comida, rol y raciones, baja con confirmación y edición de raciones, conviviendo con Valet. La planificación requiere conexión: sin red los controles se deshabilitan con aviso (no se encola) y el plan se refresca al recuperar la conexión. Objetivos táctiles ≥48px y gestión de foco (#54).

- **Recetario:** etiquetas libres de recetas (many-to-many, migración `0007`): `GET /api/tags` (`read:recipes`, con `recipe_count`), `POST /api/tags` idempotente por `slug` normalizado (`write:recipes`, `201`/`200`/`400`) y `DELETE /api/tags/{id}` en cascada sin borrar recetas (`204`/`404`); `POST`/`PUT /api/recipes` aceptan `tags` por nombre (creación al vuelo, deduplicación por `slug` y reemplazo del conjunto en la misma transacción `BEGIN IMMEDIATE`) y `GET /api/recipes?tag=<id|slug>` filtra combinado con `q`/`category`/paginación. Las etiquetas quedan **fuera** del índice `recipes_fts`. En la web: selector de etiquetas con creación al vuelo en el formulario, etiquetas en la tarjeta y el detalle, y filtro por etiqueta reflejado en la URL (`?tag=`) — todo con degradado offline y objetivos táctiles ≥48px (#55).
- **Lista de la compra:** volcado de una receta a la lista con `POST /api/shopping-list/from-recipe` (permiso `write:shopping`, migración `0008`): registra las recetas volcadas y sus raciones y **reconsolida** todas las fuentes de receta de forma declarativa e idempotente —repetir no duplica, cambiar las raciones reescala sin acumular y las líneas del plan y los ítems manuales se conservan intactos—, escalando cada cantidad por `raciones_solicitadas / raciones_receta` antes de consolidar, respetando las unidades no aditivas (`al gusto`, familias incompatibles) y desregistrando la fuente en cascada al borrar la receta. En la web: acción «Añadir a la lista» con selector de raciones (por defecto las de la receta) y enlace a `/shopping` en el detalle, deshabilitada sin conexión (no se encola) y con objetivos táctiles ≥48px (#56).

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
- Migraciones versionadas `0001_init`, `0002_fts_triggers` y `0003_fts_diacritics_and_indexes`: búsqueda FTS5 *external-content* con trigramas y plegado de diacríticos.
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
