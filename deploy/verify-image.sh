#!/usr/bin/env bash
# Verificacion automatizada de la imagen y el despliegue de OxiMeal.
# NO es un gate de CI. Requiere podman (o docker) y curl.
#
#   bash deploy/verify-image.sh
#
# Variables: RUNTIME (podman|docker), HOSTPORT (por defecto 19080).
set -uo pipefail

RUNTIME="${RUNTIME:-podman}"
IMAGE="ghcr.io/atareao/oximeal:local"
HOSTPORT="${HOSTPORT:-19080}"
BASE="http://127.0.0.1:${HOSTPORT}"
CNAME="oximeal-verify"
CNAME2="oximeal-verify-restart"
VOL="oximeal-verify-data"
BUILD_LOG="/tmp/opencode/oximeal-verify-build.log"
PASS=0
FCNT=0
ENV_CREATED=""

pass() { printf '  PASS  %s\n' "$1"; PASS=$((PASS + 1)); }
fail() { printf '  FAIL  %s\n' "$1"; FCNT=$((FCNT + 1)); }

cleanup() {
  "$RUNTIME" rm -f "$CNAME" "$CNAME2" >/dev/null 2>&1 || true
  "$RUNTIME" volume rm -f "$VOL" >/dev/null 2>&1 || true
  [ -n "$ENV_CREATED" ] && rm -f deploy/.env || true
}
trap cleanup EXIT

# Elige el binario de compose disponible.
if command -v docker-compose >/dev/null 2>&1; then COMPOSE="docker-compose"
elif command -v podman-compose >/dev/null 2>&1; then COMPOSE="podman-compose"
else COMPOSE=""; fi

# Argumentos de build: podman necesita --format docker para conservar HEALTHCHECK.
BUILD_ARGS=()
[ "$RUNTIME" = "podman" ] && BUILD_ARGS+=(--format docker)

echo "== 1. Build de la imagen ($RUNTIME) =="
mkdir -p /tmp/opencode
if timeout 3600 "$RUNTIME" build "${BUILD_ARGS[@]}" -t "$IMAGE" . >"$BUILD_LOG" 2>&1; then
  pass "build ($RUNTIME ${BUILD_ARGS[*]:-})"
else
  fail "build (ver $BUILD_LOG)"
  echo "== RESUMEN: PASS=$PASS FAIL=$FCNT =="
  exit 1
fi

echo "== 2. Metadatos de la imagen =="
USER_CFG="$("$RUNTIME" inspect --format '{{.Config.User}}' "$IMAGE" 2>/dev/null)"
[ "$USER_CFG" = "10001:10001" ] && pass "usuario no root ($USER_CFG)" || fail "usuario esperado 10001:10001, got '$USER_CFG'"

EP="$("$RUNTIME" inspect --format '{{json .Config.Entrypoint}}' "$IMAGE" 2>/dev/null)"
case "$EP" in *oximeal-backend*) pass "entrypoint ($EP)";; *) fail "entrypoint incorrecto: $EP";; esac

EXPOSE="$("$RUNTIME" inspect --format '{{.Config.ExposedPorts}}' "$IMAGE" 2>/dev/null)"
case "$EXPOSE" in *"8080/tcp"*) pass "EXPOSE 8080";; *) fail "falta EXPOSE 8080 ($EXPOSE)";; esac

VOLCFG="$("$RUNTIME" inspect --format '{{.Config.Volumes}}' "$IMAGE" 2>/dev/null)"
case "$VOLCFG" in *"/app/data"*) pass "VOLUME /app/data";; *) fail "falta VOLUME /app/data ($VOLCFG)";; esac

HEALTH="$({ "$RUNTIME" inspect "$IMAGE" 2>/dev/null || true; } | python3 -c 'import sys,json
try:
    d=json.load(sys.stdin)[0]
except Exception:
    sys.exit(0)
h=(d.get("Healthcheck") or (d.get("Config") or {}).get("Healthcheck") or {})
print(" ".join(h.get("Test",[]) or []))' 2>/dev/null || true)"
case "$HEALTH" in *"/api/health"*) pass "HEALTHCHECK contra /api/health";; *) fail "HEALTHCHECK ausente o incorrecto: [$HEALTH]";; esac

echo "== 2b. Conformidad del Dockerfile (lockfile frontend) =="
if grep -q -- '--frozen-lockfile' Dockerfile && grep -q 'pnpm-lock.yaml' Dockerfile; then
  pass "frontend usa pnpm-lock.yaml + --frozen-lockfile"
else
  fail "el stage frontend no fija el lockfile (--frozen-lockfile / pnpm-lock.yaml)"
fi

echo "== 3. .dockerignore =="
DIGNORE_OK=1
for p in "target/" "frontend/node_modules/" "frontend/dist/" "frontend/tsconfig.tsbuildinfo" ".git/" ".github/" "openspec/" "data/" ".env" "deploy/.env"; do
  grep -qxF "$p" .dockerignore || { DIGNORE_OK=0; echo "     falta en .dockerignore: $p"; }
done
[ "$DIGNORE_OK" = 1 ] && pass ".dockerignore completo" || fail ".dockerignore incompleto"
if "$RUNTIME" run --rm --entrypoint sh "$IMAGE" -c 'test ! -e /app/.env && test ! -e /app/deploy/.env && test ! -e /usr/local/cargo && echo OK' 2>/dev/null | grep -q OK; then
  pass "sin .env ni toolchain Rust en la imagen"
else
  fail "la imagen contiene .env o toolchain"
fi

echo "== 4. Arranque (modo abierto, volumen con nombre) =="
"$RUNTIME" rm -f "$CNAME" >/dev/null 2>&1 || true
"$RUNTIME" volume rm -f "$VOL" >/dev/null 2>&1 || true
if "$RUNTIME" run -d --name "$CNAME" -e OIDC_ENABLED=false \
     -p "127.0.0.1:${HOSTPORT}:8080" -v "$VOL:/app/data" "$IMAGE" >/dev/null 2>&1; then
  pass "contenedor arrancado"
else
  fail "no arranca el contenedor"; echo "== RESUMEN: PASS=$PASS FAIL=$FCNT =="; exit 1
fi

STATE=""
for _ in $(seq 1 60); do
  STATE="$("$RUNTIME" inspect --format '{{.State.Health.Status}}' "$CNAME" 2>/dev/null || echo unknown)"
  [ "$STATE" = "healthy" ] && break
  sleep 1
done
[ "$STATE" = "healthy" ] && pass "estado healthy" || fail "estado no healthy ($STATE)"
"$RUNTIME" logs "$CNAME" >/tmp/opencode/oximeal-verify-run.log 2>&1 || true

echo "== 5. Salud y usuario en ejecucion =="
HC="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/health")"
[ "$HC" = "200" ] && pass "/api/health -> 200" || fail "/api/health -> $HC"
BODY="$(curl -s "$BASE/api/health")"
case "$BODY" in *'"status":"ok"'* ) pass "cuerpo health status=ok";; * ) fail "cuerpo health inesperado: $BODY";; esac
UID_EXEC="$("$RUNTIME" exec "$CNAME" id -u 2>/dev/null)"
[ "$UID_EXEC" = "10001" ] && pass "proceso corre como UID 10001" || fail "UID del proceso = $UID_EXEC"
TC="$("$RUNTIME" exec "$CNAME" sh -c 'command -v cargo node npm pnpm 2>/dev/null || true')"
[ -z "$TC" ] && pass "sin toolchain en runtime" || fail "toolchain presente: $TC"

echo "== 6. SPA y API =="
for route in "/" "/recipes"; do
  RC="$(curl -s -o /dev/null -w '%{http_code}' "$BASE$route")"
  PAGE="$(curl -s "$BASE$route")"
  if [ "$RC" = "200" ] && printf '%s' "$PAGE" | grep -q 'id="root"'; then
    pass "SPA $route -> 200 (index.html)"
  else
    fail "SPA $route -> $RC sin index.html"
  fi
done
BAD="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/unknown")"
[ "$BAD" != "200" ] && pass "/api/unknown NO cae al fallback ($BAD)" || fail "/api/unknown devolvio 200"

echo "== 7. Persistencia entre reinicios =="
POST="$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'Content-Type: application/json' \
  -d '{"name":"leche","amount":1.0,"unit":"ud","client_id":"verify-persist-1"}' "$BASE/api/shopping-list")"
case "$POST" in 200|201) pass "item creado (HTTP $POST)";; *) fail "POST item -> $POST";; esac
"$RUNTIME" rm -f "$CNAME" >/dev/null 2>&1 || true
"$RUNTIME" run -d --name "$CNAME2" -e OIDC_ENABLED=false \
  -p "127.0.0.1:${HOSTPORT}:8080" -v "$VOL:/app/data" "$IMAGE" >/dev/null 2>&1 || true
STATE2=""
for _ in $(seq 1 60); do
  STATE2="$("$RUNTIME" inspect --format '{{.State.Health.Status}}' "$CNAME2" 2>/dev/null || echo unknown)"
  [ "$STATE2" = "healthy" ] && break
  sleep 1
done
LIST="$(curl -s "$BASE/api/shopping-list" || true)"
case "$LIST" in *leche*) pass "datos persisten tras reinicio";; *) fail "dato 'leche' no persiste";; esac

echo "== 8. Compose =="
if [ -n "$COMPOSE" ]; then
  if [ ! -f deploy/.env ]; then cp deploy/.env.example deploy/.env; ENV_CREATED=1; fi
  CFG="$("$COMPOSE" -f deploy/compose.yaml config 2>/tmp/opencode/oximeal-verify-compose.log || true)"
  if printf '%s' "$CFG" | grep -q 'oximeal' && printf '%s' "$CFG" | grep -q 'pocket-id'; then
    pass "compose valido (oximeal + pocket-id) via $COMPOSE"
  else
    fail "compose config invalido (ver /tmp/opencode/oximeal-verify-compose.log)"
  fi
  # OIDC_ENABLED debe poder activarse desde deploy/.env (compose no debe forzarlo).
  ORIG_ENV=""
  [ -f deploy/.env ] && ORIG_ENV="$(cat deploy/.env)"
  printf 'OIDC_ENABLED=true
' > deploy/.env
  CFG2="$("$COMPOSE" -f deploy/compose.yaml config 2>/dev/null || true)"
  case "$CFG2" in
    *'OIDC_ENABLED: "true"'*|*'OIDC_ENABLED: true'*) pass "OIDC_ENABLED del .env se propaga a oximeal";;
    *) fail "compose fuerza/no propaga OIDC_ENABLED";;
  esac
  if [ -n "$ORIG_ENV" ]; then printf '%s\n' "$ORIG_ENV" > deploy/.env; else rm -f deploy/.env; ENV_CREATED=""; fi
else
  fail "no hay docker-compose ni podman-compose"
fi

echo
echo "== RESUMEN: PASS=$PASS FAIL=$FCNT =="
[ "$FCNT" -eq 0 ] || exit 1
exit 0
