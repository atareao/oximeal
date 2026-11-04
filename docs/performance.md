# Rendimiento en dispositivo real

Metodología, presupuestos y resultados de la verificación de rendimiento del
binario monolítico con la UI embebida (`cargo build --release -p
oximeal-backend --features embed-ui`). Change OpenSpec
`perf-device-verification`; trazabilidad en el issue **#47**.

> La verificación es una **medición reproducible on-demand** (`just perf`), no
> un *gate* de CI (ADR-31): las métricas de rendimiento en CI son ruidosas y
> frágiles. Se ejecuta a mano sobre el binario release y se documenta aquí.

## Presupuestos

| Métrica | Presupuesto | Justificación |
|---|---|---|
| RSS estable del servidor | ≤ 16 MB | Margen de ~4 MB bajo el techo global `<20 MB` de `PLAN.md` §10 para absorber picos transitorios |
| RSS pico del servidor | ≤ 20 MB | Techo global de `PLAN.md` §10 |
| Primer contenido útil (FCP) | ≤ 1,5 s | Objetivo mobile-first en 3G razonable |
| Transferencia del grafo inicial declarado | ≤ 500 kB | Coherente con el presupuesto de bundle de `check-bundle` |
| Objetivos táctiles principales | ≥ 48 × 48 px | Accesibilidad táctil (navegación y modo compra) |

> Unidades: `check-bundle` mide **bytes en disco sin comprimir** de los chunks
> del grafo estático; `perf-web` mide **`transferSize`** (bytes de red, con
> cabeceras y, si el servidor comprimiera, ya comprimidos) de esos mismos
> recursos declarados más el `index.html`. Por eso las cifras (312 kB vs.
> ~316,7 kB) difieren ligeramente y no son intercambiables.

## Cómo reproducir

```bash
just perf          # RSS + auditoría web (encadena ui-build y el build release)
just perf-mem      # solo RSS del servidor
just perf-web      # solo auditoría web (FCP, transferencia, táctil)
```

`perf-mem`/`perf-web` dependen de `build-full`, que a su vez ejecuta
`ui-build`; no hay que compilar nada a mano. Requisitos: `curl`/`awk` para el
RSS y `google-chrome-stable` (o `CHROME_BIN`) para la auditoría web. El script
web usa `fetch`/`WebSocket` globales de Node (≥ 22), **sin dependencias nuevas**.

## Metodología

### 1. RSS del servidor — `scripts/perf-rss.sh`

1. Arranca `target/release/oximeal-backend` con `BIND_ADDR=127.0.0.1:<puerto
   libre>`, `DATABASE_URL=sqlite://<tmp>/perf.db`, `OIDC_ENABLED=false` y
   `RUST_LOG=warn`, contra una base temporal.
2. Sondea `GET /` hasta que responde.
3. Lanza una ráfaga de 30 iteraciones sobre el shell y los assets a los que
   apunta (`/assets/*.js|css`) para provocar el trabajo real (servir el HTML y
   componer los ficheros embebidos con `rust-embed`).
4. Muestrea `/proc/<pid>/status`: **`VmRSS`** (valor estable: mediana de las
   muestras recogidas durante y después de la ráfaga) y **`VmHWM`** (pico del
   proceso).
5. Sale con código ≠ 0 si el RSS estable supera 16 MB o el pico supera 20 MB,
   indicando la métrica incumplida.

### 2. Rendimiento web — `scripts/perf-web.mjs`

Auditoría por **DevTools Protocol de Chrome headless**, sin Playwright ni
dependencias:

1. El propio script arranca el binario release (o audita `APP_URL` si se le
   pasa) y lanza Chrome con `--headless=new --remote-debugging-port=<libre>`.
2. Conecta por WebSocket a `/json/version`, crea un target y emula un viewport
   móvil con `Emulation.setDeviceMetricsOverride` (390 × 844, `mobile: true`).
3. Navega a `http://127.0.0.1:<app>/` y registra el **FCP** (vía
   `PerformanceObserver` sobre `paint`) en cuanto ocurre.
4. **Transferencia del grafo inicial declarado** = `transferSize` de la
   navegación + `transferSize` de los recursos **declarados por el
   `index.html`** (entry + `<link rel=modulepreload>` + hojas de estilo),
   extraídos del HTML servido. El snapshot se toma **después de
   `Page.loadEventFired`** (con una espera acotada a que los recursos
   declarados tengan su `transferSize`), no en el instante del FCP: en ese
   momento los recursos aún en vuelo tendrían `transferSize` 0 y
   infrafacturarían. Este conjunto es el equivalente en red del grafo estático
   que mide `check-bundle`. Los chunks cargados por **imports dinámicos** en
   runtime (p. ej. antd, que `AppRoot` difiere con `React.lazy`) se miden aparte
   como *diferidos* y **no entran en el presupuesto**.
5. **Táctil**: navega a `/shopping` y evalúa `getBoundingClientRect()` de los
   controles principales (`[data-testid="bottom-nav"] button`, el enlace del
   asistente si `VALET_URL` está configurada, y `.add-item-form__submit`),
   comprobando ≥ 48 × 48 px.
6. Sale con código ≠ 0 si FCP > 1,5 s, transferencia del grafo declarado > 500 kB o algún
   objetivo táctil < 48 px.

### 3. Build release — `[profile.release]`

El `Cargo.toml` raíz fija `opt-level = 3`, `lto = true`, `codegen-units = 1`,
`panic = "abort"` y `strip = true`. Solo afecta a `--release`: `cargo test`
sigue usando el perfil `dev` (con *unwinding*), de modo que la suite no se ve
afectada.

## Resultados (medidos)

Entorno: Linux x86_64, `google-chrome-stable` headless, Node v24. Todas las
cifras se obtuvieron con `just perf` (o sus recetas) sobre el binario release
con la UI embebida.

| Métrica | Baseline (perfil por defecto) | Optimizado (`[profile.release]`) | Presupuesto | Veredicto |
|---|---:|---:|---:|:---:|
| Tamaño del binario | 24.635.264 B (23,5 MiB) | 16.716.904 B (15,9 MiB) | — | −32 % |
| RSS estable | 12,0 MB | **11,0 MB** | ≤ 16 MB | ✅ |
| RSS pico (`VmHWM`) | 12,0 MB | **11,0 MB** | ≤ 20 MB | ✅ |
| FCP | ~40 ms | **~56 ms** | ≤ 1,5 s | ✅ |
| Transferencia del grafo inicial declarado | 316,7 kB | **316,7 kB** | ≤ 500 kB | ✅ |
| Táctil navegación | 130 × 48 px | **130 × 48 px** | ≥ 48 × 48 px | ✅ |
| Táctil alta de compra | 328 × 48 px | **328 × 48 px** | ≥ 48 × 48 px | ✅ |

**Veredicto:** todos los presupuestos se cumplen con el perfil release
optimizado. El RSS estable queda **~5 MB por debajo del techo de 16 MB** (y el
pico ~9 MB por debajo de 20 MB), lo que absorbe picos transitorios. La
transferencia del grafo inicial declarado (316,7 kB) replica el grafo estático
que `check-bundle` mide en 312 kB, más el `index.html`.

### Chunks diferidos (informativos)

Fuera del presupuesto inicial, la app carga en runtime el chunk
`assets/antd-*.js` (484,6 kB) más otros módulos diferidos, que
`main.tsx`/`AppRoot` separan con `React.lazy` para no bloquear el primer
pintado. La inclusión de esos chunks asíncronos no forma parte de la
«transferencia inicial» coherente con `check-bundle`.

> Nota: si se midiera la red **completa** hasta el evento `load` (incluyendo
> antd), el total superaría los 500 kB. El presupuesto de 500 kB se define
> sobre el grafo inicial declarado, en coherencia con `check-bundle`; el
> número completo se deja registrado aquí para transparencia.

### Reproducibilidad

- El RSS del servidor es muy estable (muestras dentro de ~60 kB) y el perfil
  optimizado lo baja de 12,0 a 11,0 MB.
- El FCP en localhost es de decenas de milisegundos (limitado por CPU, no por
  red); el presupuesto de 1,5 s es amplio y está pensado para redes móviles.

## Interacciones clave con y sin conexión

El requisito **Verificación táctil** de la spec tiene tres escenarios. Este es
su estado de cobertura y trazabilidad.

### Escenario 1 — Objetivos táctiles mínimos

Los controles principales miden ≥ 48 × 48 px, medidos por `scripts/perf-web.mjs`
con viewport móvil (390 × 844) vía `getBoundingClientRect()`:

| Control | Selector | Medición |
|---|---|---|
| Navegación inferior (Menú / Recetas / Compra) | `[data-testid="bottom-nav"] button` | 130 × 48 px |
| Alta de la compra (Añadir) | `.add-item-form__submit` | 328 × 48 px |
| Asistente (si `VALET_URL`) | `[data-testid="bottom-nav"] a` | omitido sin Valet configurado |

`just perf` falla con código ≠ 0 si algún objetivo baja de 48 px. La navegación
(`bottom-nav`) y el modo compra (`add-item-form__submit`) quedan así medidos y
**ligados a los escenarios de interacción** de abajo.

### Escenario 2 — Interacciones clave con conexión

Verificado por el **smoke E2E de Fase 5** (`smoke.mjs`, 37/37): el menú se pinta
(no una pantalla 404), el Service Worker se activa y la vista de la compra carga
la lista **con conexión** (el ítem se siembra vía API con la red activa).

El alta desde el formulario **no se ejerce «con conexión»**: el smoke rellena
`#add-item-name`/`#add-item-amount` y pulsa `.add-item-form__submit` **sin
conexión** (entra en la cola offline) y luego verifica la **sincronización** al
recuperar la red. Es decir, el alta del formulario se cubre end-to-end
**offline → sincronización** (ver Escenario 3).

Nota: el smoke navega entre vistas con `page.goto(...)`, no pulsando los
botones de `bottom-nav`. El **pulsado táctil real** de la navegación no lo cubre
el smoke: queda cubierto por los objetivos ≥ 48 × 48 px medidos por
`scripts/perf-web.mjs` (Escenario 1), que mide el tamaño de los controles reales
de `bottom-nav` y del botón de alta.

### Escenario 3 — Interacciones clave sin conexión

Verificado por el mismo smoke E2E (37/37), apoyado en las capabilities base:

- **`pwa-shell`** (`openspec/specs/pwa-shell/spec.md`): el Service Worker sirve
  el shell desde la caché al recargar sin conexión (*network-first* con fallback
  al `index.html` cacheado). El smoke comprueba `navigator.serviceWorker.controller`,
  las cachés creadas y el precache poblado (> 5 entradas).
- **`offline-data`** (`openspec/specs/offline-data/spec.md`): la lista de la
  compra se muestra desde IndexedDB sin conexión y aparece el aviso de modo
  degradado `[data-testid="offline-banner"]`.
- **`offline-writes`** (`openspec/specs/offline-writes/spec.md`): el alta sin
  conexión se aplica de forma optimista, se marca como pendiente
  `[data-testid="item-pending"]` y encolada, con el aviso
  `[data-testid="pending-mutations"]`; la cola **persiste tras recargar** sin
  conexión. Al recuperar la conexión se **drena** y el ítem llega al servidor en
  **un único POST** con su `client_id` (idempotente: reenviar el mismo
  `client_id` devuelve 200 sin duplicar) y la lista deja de mostrar pendientes.

Comprobaciones offline del smoke (extracto): «shell arranca sin conexión (SW
sirve index.html)», «lista visible sin conexión desde la caché», «aviso de modo
degradado/offline visible», «el ítem sin sincronizar se marca como pendiente»,
«aviso de cambios pendientes», «la cola persiste tras recargar sin conexión»,
«la cola se drena al recuperar la conexión», «un único POST de alta durante el
drenado», «el ítem sincronizado guarda su `client_id`».

### Cómo reproducir las interacciones offline

El harness del smoke usa **Playwright**, que **no es una dependencia del repo**
(la herramienta de rendimiento `perf-web.mjs` usa CDP sin dependencias). Por eso
`smoke.mjs` **no se versiona aquí**: es un **arnés de verificación externo**
(`/tmp/opencode/smoke/smoke.mjs`, con su `proxy.mjs` y su `node_modules`).
Resultado medido sobre el binario release optimizado: **37/37**.

```bash
# 1. Binario release con la UI embebida sobre una base temporal.
BIND_ADDR=127.0.0.1:8099 DATABASE_URL=sqlite:///tmp/smoke.db \
  OIDC_ENABLED=false RUST_LOG=warn \
  ./target/release/oximeal-backend &

# 2. Proxy de verificación (fuerza el 404 JSON de /auth/* del modo abierto).
node /tmp/opencode/smoke/proxy.mjs &      # escucha en :8100 → :8099

# 3. Smoke E2E (Playwright del arnés externo).
cd /tmp/opencode/smoke && BASE=http://127.0.0.1:8100 node smoke.mjs
# → 37/37 comprobaciones OK
```

Pasos equivalentes **sin Playwright** (reproducibles con el mismo CDP que usa
`perf-web.mjs`): lanzar Chrome headless con `--remote-debugging-port`, crear un
target, `Network.enable` y `Network.setOffline({ offline: true })`; recargar y
comprobar `navigator.serviceWorker.controller`, `caches.keys()` y el
`index.html` servido desde caché, la visibilidad de la lista y
`[data-testid="offline-banner"]`; rellenar `#add-item-name`/`#add-item-amount`,
pulsar `.add-item-form__submit` y verificar `[data-testid="item-pending"]` y
`[data-testid="pending-mutations"]`; volver a `Network.setOffline({ offline: false })`,
disparar el evento `online` y comprobar que la cola se drena (un solo POST a
`/api/shopping-list` con `client_id`) y que los avisos desaparecen.

## Trazabilidad

- Change: `openspec/changes/perf-device-verification/`.
- Issue: **#47**.
- ADR-31: verificación de rendimiento on-demand, no como *gate* de CI.
- `PLAN.md` §10: objetivo global `<20 MB` verificado sobre el binario
  monolítico.
