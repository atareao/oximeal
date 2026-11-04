#!/usr/bin/env bash
#
# perf-rss.sh — mide el RSS del binario release con la UI embebida.
#
# No es un gate de CI (ADR-31): es una verificación de rendimiento manual,
# reproducible con `just perf` / `just perf-mem`.
#
# Metodología:
#   1. Arranca `target/release/oximeal-backend` (compilado con `--features
#      embed-ui`) contra una base de datos temporal.
#   2. Sondea `GET /` hasta que el servidor responde.
#   3. Lanza una ráfaga de peticiones al shell y a los assets a los que apunta
#      (`/assets/*.js|css`) para provocar el trabajo real: servir el HTML y
#      descomprimir/componer los ficheros embebidos.
#   4. Muestrea `/proc/<pid>/status`: `VmRSS` (valor estable, mediana de las
#      muestras) y `VmHWM` (pico del proceso).
#   5. Sale con código ≠ 0 si el RSS estable > 16 MB o el pico > 20 MB.
#
# Requisitos: bash, curl, awk. El binario debe existir (lo compila la receta
# `just perf-mem`).
#
# Variables de entorno opcionales:
#   RSS_STABLE_MB   Umbral de RSS estable en MB (por defecto 16).
#   RSS_PEAK_MB     Umbral de RSS pico en MB (por defecto 20).
#   BURST           Número de iteraciones de la ráfaga (por defecto 30).

set -euo pipefail

STABLE_BUDGET_MB="${RSS_STABLE_MB:-16}"
PEAK_BUDGET_MB="${RSS_PEAK_MB:-20}"
BURST="${BURST:-30}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$REPO_ROOT/target/release/oximeal-backend"

if [[ ! -x "$BIN" ]]; then
    echo "❌ No existe el binario release: $BIN" >&2
    echo "   Compílalo con: just build-full  (o usa: just perf-mem)" >&2
    exit 1
fi

for tool in curl awk; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "❌ Falta la herramienta requerida: $tool" >&2
        exit 1
    fi
done

# --- Utilidades -----------------------------------------------------------

# Devuelve un puerto TCP libre en 127.0.0.1 (probando con /dev/tcp).
free_port() {
    local port
    for _ in $(seq 1 50); do
        port=$(( (RANDOM % 20000) + 20000 ))
        if ! (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
            echo "$port"
            return 0
        fi
    done
    echo "No se encontró un puerto libre" >&2
    return 1
}

# Lee un campo de /proc/<pid>/status (en kB) y lo imprime.
read_status_kb() {
    local pid="$1" key="$2"
    awk -v k="$key:" '$1 == k { print $2; exit }' "/proc/$pid/status" 2>/dev/null
}

# Mediana (en kB) de una lista de números pasada por stdin.
median() {
    sort -n | awk '{ v[NR]=$1 } END {
        if (NR == 0) { print 0; exit }
        if (NR % 2) { printf "%.0f\n", v[(NR + 1) / 2] }
        else { printf "%.0f\n", (v[NR / 2] + v[NR / 2 + 1]) / 2 }
    }'
}

# --- Preparación ----------------------------------------------------------

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/oximeal-perf-rss.XXXXXX")"
PID=""
PORT=""
BASE_URL=""

cleanup() {
    if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
        kill "$PID" 2>/dev/null || true
        # Espera breve a que muera; si no, fuerza.
        for _ in $(seq 1 20); do
            kill -0 "$PID" 2>/dev/null || break
            sleep 0.1
        done
        kill -9 "$PID" 2>/dev/null || true
    fi
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# --- Arranque con reintento ante EADDRINUSE -------------------------------
#
# `free_port` comprueba con `/dev/tcp` y libera el puerto antes de que el
# binario haga `bind`: entre ambos hay una ventana TOCTOU en la que otro
# proceso podría ocuparlo. Si el arranque falla por `EADDRINUSE`, se reintenta
# con un puerto nuevo (hasta `MAX_ATTEMPTS`).

MAX_ATTEMPTS=5
ready=0
for attempt in $(seq 1 "$MAX_ATTEMPTS"); do
    PORT="$(free_port)"
    BASE_URL="http://127.0.0.1:$PORT"

    echo "▶ Intento $attempt/$MAX_ATTEMPTS: arrancando $BIN en $BASE_URL (BD temporal: $TMP_DIR/perf.db)"

    BIND_ADDR="127.0.0.1:$PORT" \
    DATABASE_URL="sqlite://$TMP_DIR/perf.db" \
    OIDC_ENABLED=false \
    RUST_LOG=warn \
        "$BIN" >"$TMP_DIR/server.log" 2>&1 &
    PID=$!

    for _ in $(seq 1 100); do
        if ! kill -0 "$PID" 2>/dev/null; then
            break
        fi
        if curl -fsS -o /dev/null "$BASE_URL/" 2>/dev/null; then
            ready=1
            break
        fi
        sleep 0.2
    done

    if [[ "$ready" -eq 1 ]]; then
        break
    fi

    # Falló el arranque: ¿es un puerto ocupado? Si sí, reintenta; si no, aborta.
    if grep -q "Address already in use\|EADDRINUSE" "$TMP_DIR/server.log" 2>/dev/null; then
        echo "⚠️  Puerto $PORT ocupado (EADDRINUSE); reintentando con otro puerto." >&2
        kill "$PID" 2>/dev/null || true
        PID=""
        continue
    fi

    echo "❌ El servidor no respondió a GET / en el tiempo esperado. Log:" >&2
    cat "$TMP_DIR/server.log" >&2 || true
    exit 1
done

if [[ "$ready" -ne 1 ]]; then
    echo "❌ No se pudo arrancar el servidor tras $MAX_ATTEMPTS intentos. Log:" >&2
    cat "$TMP_DIR/server.log" >&2 || true
    exit 1
fi

echo "▶ Servidor listo (pid $PID, RSS inicial $(( $(read_status_kb "$PID" VmRSS 2>/dev/null || echo 0) / 1024 )) MB)"

# --- Recolectar rutas del shell (index + assets) --------------------------

INDEX_HTML="$(curl -fsS "$BASE_URL/" 2>/dev/null || true)"

# Rutas de assets referenciadas en el HTML: src="/..." y href="/..." de JS/CSS.
mapfile -t ASSET_PATHS < <(
    printf '%s' "$INDEX_HTML" \
        | grep -oE '(src|href)="/[^"]+\.(js|css)"' \
        | sed -E 's/^(src|href)="//; s/"$//' \
        | sort -u
)

PATHS=("/" "${ASSET_PATHS[@]}")
echo "▶ Ráfaga de $BURST iteraciones sobre ${#PATHS[@]} rutas (index + assets)"

# --- Ráfaga + muestreo -----------------------------------------------------

RSS_SAMPLES=""
# Muestra de calentamiento (tras la primera petición).
for path in "${PATHS[@]}"; do
    curl -fsS -o /dev/null "$BASE_URL$path" 2>/dev/null || true
done

for _ in $(seq 1 "$BURST"); do
    for path in "${PATHS[@]}"; do
        curl -fsS -o /dev/null "$BASE_URL$path" 2>/dev/null || true
    done
    rss="$(read_status_kb "$PID" VmRSS 2>/dev/null || echo 0)"
    if [[ "$rss" -gt 0 ]]; then
        RSS_SAMPLES+="$rss"$'\n'
    fi
done

# Muestras de reposo tras la ráfaga (para el valor estable).
for _ in $(seq 1 5); do
    sleep 0.2
    rss="$(read_status_kb "$PID" VmRSS 2>/dev/null || echo 0)"
    [[ "$rss" -gt 0 ]] && RSS_SAMPLES+="$rss"$'\n'
done

HWM_KB="$(read_status_kb "$PID" VmHWM 2>/dev/null || echo 0)"

if [[ -z "$RSS_SAMPLES" ]]; then
    echo "❌ No se pudo leer /proc/$PID/status (¿el proceso terminó?)" >&2
    cat "$TMP_DIR/server.log" >&2 || true
    exit 1
fi

STABLE_KB="$(printf '%s' "$RSS_SAMPLES" | median)"
STABLE_MB="$(awk -v kb="$STABLE_KB" 'BEGIN { printf "%.1f", kb / 1024 }')"
PEAK_MB="$(awk -v kb="$HWM_KB" 'BEGIN { printf "%.1f", kb / 1024 }')"

echo "──────────────────────────────────────────────"
echo "RSS muestras (kB): $(printf '%s' "$RSS_SAMPLES" | tr '\n' ' ')"
echo "estable=$STABLE_MB MB pico=$PEAK_MB MB"
echo "presupuesto: estable ≤ $STABLE_BUDGET_MB MB · pico ≤ $PEAK_BUDGET_MB MB"
echo "──────────────────────────────────────────────"

status=0
if awk -v v="$STABLE_MB" -v t="$STABLE_BUDGET_MB" 'BEGIN { exit !(v > t) }'; then
    echo "❌ Incumplido: RSS estable $STABLE_MB MB > $STABLE_BUDGET_MB MB" >&2
    status=1
fi
if awk -v v="$PEAK_MB" -v t="$PEAK_BUDGET_MB" 'BEGIN { exit !(v > t) }'; then
    echo "❌ Incumplido: RSS pico $PEAK_MB MB > $PEAK_BUDGET_MB MB" >&2
    status=1
fi

if [[ "$status" -eq 0 ]]; then
    echo "✅ RSS dentro del presupuesto (estable $STABLE_MB MB · pico $PEAK_MB MB)"
fi

exit "$status"
