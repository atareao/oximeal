# deployment Specification

## Purpose
Esta capability define el empaquetado y despliegue de OxiMeal como **imagen de
contenedor** reproducible: un `Dockerfile` multi-stage (frontend Node/pnpm →
builder Rust → runtime `debian:bookworm-slim`) que conserva el modelo de
**binario único** con la UI embebida (`--features embed-ui`), ejecuta como
usuario no root (`oximeal`, UID/GID 10001), persiste la base SQLite en el volumen
`/app/data`, declara un `HEALTHCHECK` contra `/api/health` y se configura por
entorno sin recompilar. Incluye la orquestación con `deploy/compose.yaml` junto a
Pocket ID y la documentación de despliegue (build, variables, volúmenes y
permisos, puerto, modo abierto vs OIDC y TLS/reverse-proxy) en `README.md` y
`deploy/README.md` (ADR-32).

## Requirements

### Requirement: Imagen multi-stage reproducible

El repositorio SHALL incluir un `Dockerfile` en la raíz (contexto de build = raíz
del monorepo) con **tres stages**: `frontend` (Node + pnpm), `builder` (Rust) y
`runtime` (imagen mínima). La imagen SHALL construirse como
`ghcr.io/atareao/oximeal` (etiqueta por versión/commit).

#### Scenario: Build completo de la imagen

- GIVEN un checkout limpio del repositorio con `Cargo.lock` y `frontend/pnpm-lock.yaml`
- WHEN se ejecuta `docker build -t ghcr.io/atareao/oximeal:local .`
- THEN el stage `frontend` instala con `pnpm install --frozen-lockfile` desde `frontend/pnpm-lock.yaml` y construye `frontend/dist` con `pnpm build`
- AND el stage `builder` compila el binario release con la UI embebida mediante `cargo build --release -p oximeal-backend --features embed-ui`, copiando `frontend/dist` desde el stage `frontend`
- AND el stage `runtime` contiene únicamente el binario y sus dependencias de sistema
- AND el build termina con código 0

#### Scenario: Build reproducible con lockfiles

- GIVEN los lockfiles versionados `Cargo.lock` y `frontend/pnpm-lock.yaml`
- WHEN se repite el build sin modificar el código
- THEN la instalación de pnpm usa exactamente las versiones del lockfile
- AND la compilación de Cargo respeta las versiones de `Cargo.lock`
- AND no se descargan dependencias fuera de los lockfiles

#### Scenario: Falta de lockfile

- GIVEN un checkout sin `frontend/pnpm-lock.yaml` o sin `Cargo.lock`
- WHEN se intenta construir la imagen
- THEN `pnpm install --frozen-lockfile` falla y el build se detiene
- AND no se genera una imagen parcial silenciosa

### Requirement: Cache de capas

El `Dockerfile` SHALL copiar primero los manifiestos (`Cargo.toml`, `Cargo.lock`,
`backend/Cargo.toml`, `frontend/package.json`, `frontend/pnpm-lock.yaml`) y
resolver dependencias antes de copiar el código fuente, de modo que los cambios de
código no invaliden la capa de dependencias.

#### Scenario: Cambio de código reutiliza dependencias

- GIVEN una imagen ya construida
- WHEN se modifica un fichero fuente del backend o del frontend y se reconstruye
- THEN las capas de instalación de dependencias se reutilizan desde la caché
- AND solo se recompila lo afectado por el cambio

#### Scenario: Cambio de dependencias invalida la caché

- GIVEN una imagen ya construida
- WHEN se modifica `Cargo.lock` o `frontend/pnpm-lock.yaml`
- THEN la capa de dependencias correspondiente se reconstruye
- AND el binario final se recompila con las nuevas versiones

### Requirement: `.dockerignore`

El repositorio SHALL incluir un `.dockerignore` en la raíz que excluya del
contexto de build los artefactos y secretos: `target/`, `frontend/node_modules/`,
`frontend/dist/`, `frontend/tsconfig.tsbuildinfo`, `.git/`, `.github/`,
`openspec/`, `data/`, `.env` y `deploy/.env`.

#### Scenario: El contexto no arrastra artefactos

- GIVEN un árbol de trabajo con `target/`, `frontend/node_modules/` y `data/` presentes
- WHEN se inspecciona el contexto enviado al daemon durante el build
- THEN esos directorios no forman parte del contexto
- AND el tiempo de transferencia del contexto se mantiene acotado

#### Scenario: Los secretos no entran en la imagen

- GIVEN un fichero `.env` y `deploy/.env` con secretos reales en el árbol de trabajo
- WHEN se construye la imagen
- THEN ningún fichero `.env` se copia a ningún stage
- AND los secretos no quedan embebidos en la imagen final

### Requirement: Runtime mínimo y usuario no root

El stage `runtime` SHALL basarse en `debian:bookworm-slim` e incluir únicamente
`ca-certificates` (TLS hacia el IdP OIDC) y `curl` (healthcheck). SHALL crear un
usuario/grupo no privilegiado `oximeal` (UID/GID 10001), fijar `WORKDIR /app`,
declarar `EXPOSE 8080` y configurar
`ENTRYPOINT ["/usr/local/bin/oximeal-backend"]`, sin ejecutar como `root`.

#### Scenario: Ejecución como usuario no root

- GIVEN la imagen construida
- WHEN se inspecciona el usuario configurado y el proceso en ejecución
- THEN el contenedor arranca con el usuario `oximeal` (UID 10001)
- AND el proceso no corre como `root`

#### Scenario: Binario único sin toolchain

- GIVEN la imagen final
- WHEN se inspecciona su sistema de ficheros
- THEN contiene el binario `oximeal-backend` en `/usr/local/bin`
- AND no contiene el toolchain de Rust, Node ni pnpm
- AND no expone dependencias de compilación

#### Scenario: La UI embebida se sirve desde el mismo puerto

- GIVEN un contenedor en ejecución
- WHEN se solicita `/` y una ruta del SPA (p. ej. `/recipes`)
- THEN el binario sirve la UI embebida con el fallback SPA
- AND las rutas `/api/*` no caen al fallback

#### Scenario: Se descarta distroless

- GIVEN la elección de imagen base del stage `runtime`
- WHEN se valora `distroless`
- THEN se descarta porque no aporta shell ni cliente HTTP
- AND el `HEALTHCHECK` contra `/api/health` requeriría un binario de sonda adicional

### Requirement: Persistencia de datos en volumen

La imagen SHALL declarar `VOLUME /app/data` y por defecto
`DATABASE_URL=sqlite://data/oximeal.db` (relativo a `WORKDIR /app`). El
`Dockerfile` SHALL crear `/app/data` con permisos de escritura para el usuario
`oximeal`. La base SQLite, su `-wal`, `-shm` y las migraciones SHALL persistir en
ese volumen.

#### Scenario: Datos persistentes entre reinicios

- GIVEN un contenedor con el volumen `/app/data` montado
- WHEN se crean recetas y se reinicia el contenedor
- THEN los datos siguen presentes tras el reinicio
- AND las migraciones ya aplicadas no se reejecutan de forma destructiva

#### Scenario: Volumen con permisos incorrectos

- GIVEN un bind mount en `/app/data` cuyo propietario no es el UID 10001
- WHEN el contenedor arranca
- THEN la aplicación falla al abrir/escribir la base de datos y lo registra
- AND la documentación indica cómo ajustar el propietario del volumen (`chown 10001`)

#### Scenario: Directorio de datos inexistente

- GIVEN una `DATABASE_URL` que apunta a un directorio que no existe
- WHEN el contenedor arranca
- THEN el arranque falla con un error claro de directorio/BD
- AND la documentación indica montar un volumen en `/app/data` o corregir `DATABASE_URL`

### Requirement: Healthcheck

La imagen SHALL declarar un `HEALTHCHECK` que consulte `GET /api/health` en
`http://127.0.0.1:8080/api/health` mediante `curl`, con intervalo, timeout,
periodo de arranque y reintentos definidos. El contenedor SHALL considerarse
`sano` solo cuando el endpoint responde 2xx.

#### Scenario: Contenedor sano

- GIVEN un contenedor arrancado y con la base de datos accesible
- WHEN Docker ejecuta el healthcheck
- THEN `GET /api/health` responde 200
- AND el estado del contenedor es `healthy`

#### Scenario: Base de datos no disponible

- GIVEN un contenedor cuyo volumen de datos no es accesible
- WHEN se ejecuta el healthcheck
- THEN la petición a `/api/health` no devuelve 2xx
- AND el contenedor no se marca como `healthy`

#### Scenario: Orquestador espera al arranque

- GIVEN un arranque con `start_period` configurado
- WHEN la aplicación aún no ha terminado de migrar
- THEN los fallos de healthcheck durante el `start_period` no reinician el contenedor
- AND una vez listo, el healthcheck pasa a 2xx

### Requirement: Configuración por entorno

La imagen SHALL ser configurable por variables de entorno sin recompilación,
respetando `.env.example`. SHALL documentarse el **modo abierto**
(`OIDC_ENABLED=false`, solo desarrollo), el **modo OIDC**, el endurecimiento
de sesión (`__Host-`, `SESSION_COOKIE_SECURE`, `SESSION_ABSOLUTE_TTL_SECS`,
`ALLOW_INSECURE_COOKIE`) y que HSTS y `Referrer-Policy` los aporta el *reverse
proxy* que termina el TLS.

#### Scenario: Variables de entorno reconocidas

- GIVEN el fichero `.env.example`
- WHEN se configura el contenedor
- THEN se reconocen `DATABASE_URL`, `BIND_ADDR`, `RUST_LOG`, el bloque OIDC (`OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `OIDC_ALLOWED_GROUPS`), `ALLOWED_ORIGINS`, `IMPORT_ALLOWED_HOSTS` y `SESSION_*`
- AND ninguna de ellas exige recompilar la imagen

#### Scenario: Modo abierto

- GIVEN un contenedor con `OIDC_ENABLED=false`
- WHEN la aplicación arranca sin variables OIDC
- THEN la API queda accesible sin autenticación y lo advierte en logs
- AND el contenedor responde en `/api/health`

#### Scenario: Modo OIDC con configuración incompleta

- GIVEN un contenedor con `OIDC_ENABLED=true` y falta una variable OIDC obligatoria
- WHEN la aplicación arranca
- THEN el proceso falla al arrancar con un error explícito
- AND el healthcheck permanece en fallo (el contenedor no se marca `healthy`)

#### Scenario: Terminación TLS en el proxy

- GIVEN el despliegue recomendado con el TLS terminado en el *reverse proxy*
- WHEN se documenta en `deploy/README.md`
- THEN se indica que la cookie viaja con `Secure` y prefijo `__Host-`
- AND se documenta que HSTS y `Referrer-Policy` se configuran en el proxy, no en la aplicación

### Requirement: Orquestación con compose

`deploy/compose.yaml` SHALL mantener el servicio `pocket-id` y SHALL añadir un
servicio `oximeal` que use la imagen construida localmente o `ghcr.io/atareao/oximeal`,
declare `env_file`, monte un volumen con nombre en `/app/data`, se conecte a la red
`devnet` y exponga el puerto `8080`. El fichero SHALL seguir siendo compatible con
`podman compose` y `docker compose`. SHALL documentarse el arranque conjunto (app +
IdP) y el arranque en modo abierto (solo app).

#### Scenario: Arranque conjunto app + IdP

- GIVEN `deploy/.env` configurado con `OIDC_ENABLED=true` y los datos del IdP
- WHEN se ejecuta `docker compose -f deploy/compose.yaml up -d`
- THEN se levantan `pocket-id` y `oximeal` en la red `devnet`
- AND `oximeal` persiste la base SQLite en su volumen y queda `healthy`

#### Scenario: Arranque en modo abierto

- GIVEN `OIDC_ENABLED=false`
- WHEN se levanta `oximeal` sin dependencia del IdP
- THEN la aplicación arranca y sirve la API/UI en el puerto 8080
- AND `pocket-id` no es necesario para el funcionamiento

#### Scenario: Puerto ocupado

- GIVEN otro proceso usando el puerto 8080 en el host
- WHEN se levanta el servicio `oximeal`
- THEN el arranque falla con un error de puerto en uso
- AND la documentación indica cómo cambiar el mapeo de puertos

### Requirement: Documentación de despliegue

El repositorio SHALL documentar el despliegue en `README.md` y `deploy/README.md`:
construcción de la imagen, variables de entorno, volúmenes y permisos,
`HEALTHCHECK`, puerto, el modo abierto vs OIDC y el uso de TLS/reverse-proxy. El
`README.md` SHALL actualizar su estado a Fase 6/7 (recetario completo y entrega) y
eliminar las referencias obsoletas a «Fase 1» y al `frontend/` «reservado para la
Fase 2».

#### Scenario: Guía de build y ejecución

- GIVEN un operador nuevo
- WHEN lee la sección de despliegue del `README.md` o `deploy/README.md`
- THEN encuentra los comandos para construir la imagen y levantar el contenedor
- AND conoce las variables de entorno, el volumen de datos y el puerto 8080

#### Scenario: Guía de TLS y reverse-proxy

- GIVEN un despliegue fuera de `localhost` con OIDC activo
- WHEN el operador consulta la documentación
- THEN se explica que el IdP exige HTTPS en los redirect URIs
- AND se indica servir OxiMeal tras un reverse-proxy TLS (p. ej. Caddy/Traefik/nginx)

#### Scenario: Estado del proyecto actualizado

- GIVEN el `README.md`
- WHEN se revisa su sección de estado/estructura
- THEN refleja la Fase 6/7 (recetario completo y entrega)
- AND no contiene referencias a «Fase 1» ni al `frontend/` como placeholder de la Fase 2
