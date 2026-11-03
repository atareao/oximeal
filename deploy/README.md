# Guia de despliegue de OxiMeal

OxiMeal se distribuye como **imagen de contenedor** con la UI React embebida en el
propio binario (Axum + SQLite). Esta guia cubre la construccion de la imagen,
el arranque, la configuracion, la persistencia y el despliegue tras un
reverse-proxy TLS.

## 1. Construir la imagen

Desde la raiz del repositorio (contexto de build = monorepo):

```bash
podman build --format docker -t ghcr.io/atareao/oximeal:local .
# o con Docker:
docker build -t ghcr.io/atareao/oximeal:local .
```

> **Nota (Podman):** Podman usa por defecto el formato OCI, que **descarta el
> `HEALTHCHECK`**. Para conservarlo hay que construir con `--format docker`
> (Docker ya usa ese formato por defecto).

El `Dockerfile` es multi-stage: `frontend` (Node + pnpm, genera `frontend/dist`),
`builder` (`rust:bookworm`, compila `oximeal-backend --release --features
embed-ui`) y `runtime` (`debian:bookworm-slim`, usuario no root).

## 2. Arrancar el contenedor (modo abierto)

```bash
podman volume create oximeal-data
podman run --name oximeal -d \
  -p 8080:8080 \
  -v oximeal-data:/app/data \
  -e OIDC_ENABLED=false \
  ghcr.io/atareao/oximeal:local
```

La app queda en <http://localhost:8080> y alcanza el estado `healthy` en unos
segundos (healthcheck contra `GET /api/health`).

## 3. Variables de entorno

Todas las variables se documentan en `deploy/.env.example` (copia a
deploy/.env). No requieren recompilar la imagen.

| Grupo | Variables |
|---|---|
| Base de datos | `DATABASE_URL` |
| Servidor | `BIND_ADDR`, `RUST_LOG` |
| OIDC | `OIDC_ENABLED`, `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_API_AUDIENCE`, `OIDC_REDIRECT_URI`, `OIDC_POST_LOGOUT_REDIRECT_URI`, `OIDC_ALLOWED_GROUPS` |
| CORS / import | `ALLOWED_ORIGINS`, `IMPORT_ALLOWED_HOSTS` |
| Sesion | `SESSION_SECRET`, `SESSION_TTL_SECS`, `SESSION_REFRESH_MARGIN_SECS`, `SESSION_COOKIE_NAME`, `SESSION_COOKIE_SECURE` |

- **Modo abierto** (`OIDC_ENABLED=false`): solo desarrollo; la API queda sin
  autenticacion y se advierte en los logs.
- **Modo OIDC** (`OIDC_ENABLED=true`): con la configuracion obligatoria arriba.
  El contenedor **falla al arrancar** si falta alguna variable o si
  `SESSION_SECRET` es inseguro.

## 4. Volumen y permisos

La imagen declara `VOLUME /app/data` y usa por defecto
`DATABASE_URL=sqlite://data/oximeal.db` (relativo a `WORKDIR /app`). La base
SQLite (`oximeal.db`, `-wal`, `-shm`) y las migraciones persisten en
`/app/data`.

El contenedor corre como el usuario **`oximeal` (UID/GID 10001)**. Usa un volumen
con nombre (recomendado) o un *bind mount* cuyo propietario sea el UID 10001:
 
```bash
# Volumen con nombre (permisos correctos automaticamente)
podman volume create oximeal-data

# Bind mount: ajusta el propietario al UID/GID 10001 del contenedor
mkdir -p /srv/oximeal-data
sudo chown -R 10001:10001 /srv/oximeal-data
podman run ... -v /srv/oximeal-data:/app/data ...
```

Si el volumen no es escribible por el UID 10001, la aplicacion falla al abrir la
base de datos y lo registra en los logs.

## 5. Healthcheck y puerto

La imagen declara un `HEALTHCHECK` que consulta
`GET http://127.0.0.1:8080/api/health` (intervalo 30 s, timeout 10 s,
`start_period` 30 s, 3 reintentos). El contenedor se marca `healthy` **solo**
cuando el endpoint responde 2xx (lo que implica que la base de datos responde).
El servicio escucha en el puerto **8080**; si esta ocupado en el host, cambia el
mapeo (`-p 8081:8080`).

## 6. Orquestacion con compose

`deploy/compose.yaml` levanta OxiMeal junto a su IdP de desarrollo (Pocket ID).

- **App + IdP (modo OIDC):** copia `deploy/.env.example` a `deploy/.env`, pon
  `OIDC_ENABLED=true`, rellena los datos del IdP y ejecuta:

  ```bash
  podman compose -f deploy/compose.yaml up -d   # o: docker compose -f deploy/compose.yaml up -d
  ```

- **Solo la app (modo abierto):** con `OIDC_ENABLED=false`, `pocket-id` no es
  necesario:

  ```bash
  podman compose -f deploy/compose.yaml up -d oximeal
  ```

Pocket ID escucha en el puerto 1411; OxiMeal en 8080.

## 7. TLS y reverse-proxy

Pocket ID **exige HTTPS en los redirect URIs** fuera de `localhost`. En
despliegues reales, sirve OxiMeal tras un **reverse-proxy TLS** (Caddy, Traefik,
nginx...) que redirija `/` y `/api/*` al puerto 8080 del contenedor, y registra
los redirect URIs con `https://`.

Ejemplo minimo con Caddy:

```
oximeal.example.com {
    reverse_proxy 127.0.0.1:8080
}
```

## 8. Verificacion automatizada

`deploy/verify-image.sh` construye la imagen y comprueba de extremo a extremo
build, metadatos, usuario no root, healthcheck, servido de la SPA, `/api/*` sin
fallback, persistencia entre reinicios y la validez del `compose`. No es un
*gate* de CI:

```bash
bash deploy/verify-image.sh
```