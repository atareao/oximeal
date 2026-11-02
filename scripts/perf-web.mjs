#!/usr/bin/env node
//
// perf-web.mjs — auditoría de rendimiento web del shell OxiMeal por CDP.
//
// Sin dependencias: usa `fetch` y `WebSocket` globales de Node ≥22 y el
// protocolo DevTools de Chrome. No es un gate de CI (ADR-31): es una
// verificación manual reproducible con `just perf` / `just perf-web`.
//
// Mide sobre `http://127.0.0.1`:
//   • FCP (first-contentful-paint) vía Performance API.
//   • Peso de la transferencia inicial: navegación + recursos `script`/`link`/
//     `css`/`fetch` (chunks) con `transferSize`.
//   • Objetivos táctiles ≥48×48 px con viewport móvil 390×844 en los controles
//     principales (navegación inferior y alta de la compra).
//
// Presupuestos: FCP ≤1500 ms · transferencia ≤500 kB · táctil ≥48×48 px.
//
// El script arranca él mismo el binario release con la UI embebida, salvo que
// se le pase `APP_URL` (entonces audita ese servidor ya arrancado).
//
// Variables de entorno:
//   APP_URL        Servidor a auditar (si falta, se arranca el binario local).
//   CHROME_BIN     Binario de Chrome (por defecto /usr/bin/google-chrome-stable).
//   FCP_BUDGET_MS  Presupuesto de FCP en ms (por defecto 1500).
//   TRANSFER_BUDGET_BYTES  Presupuesto de transferencia en bytes (por defecto 512000).
//   TOUCH_MIN_PX   Objetivo táctil mínimo en px (por defecto 48).

import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync, existsSync, openSync, closeSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = path.join(REPO_ROOT, 'target', 'release', 'oximeal-backend');
const CHROME_BIN = process.env.CHROME_BIN || '/usr/bin/google-chrome-stable';

const FCP_BUDGET_MS = Number(process.env.FCP_BUDGET_MS || 1500);
const TRANSFER_BUDGET_BYTES = Number(process.env.TRANSFER_BUDGET_BYTES || 500 * 1024);
const TOUCH_MIN_PX = Number(process.env.TOUCH_MIN_PX || 48);

const GLOBAL_TIMEOUT_MS = 120_000;
const STEP_TIMEOUT_MS = 30_000;

// ---------------------------------------------------------------------------
// Utilidades
// ---------------------------------------------------------------------------

// Reserva un puerto TCP libre manteniendo el `net.Server` ABierto hasta que se
// llame a `release()`. Así se evita la ventana TOCTOU entre "comprobar puerto
// libre" y "el hijo hace bind": el puerto se libera justo antes del `spawn`.
function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      let released = false;
      const release = () =>
        new Promise((res) => {
          if (released) return res();
          released = true;
          server.close(() => res());
        });
      resolve({ port, release });
    });
  });
}

/** Lee un fichero de log (best-effort) para volcar diagnósticos de arranque. */
function readLog(file) {
  try {
    return readFileSync(file, 'utf8').trim();
  } catch {
    return '';
  }
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Reintenta `fn` hasta que devuelva un valor `true`-ish o se agote el tiempo. */
async function waitFor(fn, { timeoutMs = STEP_TIMEOUT_MS, intervalMs = 150, label = 'condición' } = {}) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const value = await fn();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await sleep(intervalMs);
  }
  throw new Error(`Timeout esperando ${label}${lastError ? `: ${lastError.message}` : ''}`);
}

// ---------------------------------------------------------------------------
// Cliente mínimo del DevTools Protocol sobre WebSocket
// ---------------------------------------------------------------------------

class Cdp {
  constructor(ws) {
    this.ws = ws;
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Map();
    ws.addEventListener('message', (event) => {
      const msg = JSON.parse(event.data);
      if (msg.id != null) {
        const pending = this.pending.get(msg.id);
        if (!pending) return;
        this.pending.delete(msg.id);
        if (msg.error) pending.reject(new Error(`${msg.error.message} (${msg.error.code})`));
        else pending.resolve(msg.result);
        return;
      }
      if (msg.method) {
        const handlers = this.listeners.get(msg.method);
        if (handlers) for (const handler of [...handlers]) handler(msg.params, msg.sessionId);
      }
    });
  }

  static connect(url) {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(url);
      const onError = () => reject(new Error(`no se pudo conectar al WebSocket: ${url}`));
      ws.addEventListener('open', () => {
        ws.removeEventListener('error', onError);
        resolve(new Cdp(ws));
      });
      ws.addEventListener('error', onError);
    });
  }

  send(method, params = {}, sessionId) {
    const id = this.nextId++;
    const payload = { id, method, params };
    if (sessionId) payload.sessionId = sessionId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify(payload));
    });
  }

  on(method, handler) {
    if (!this.listeners.has(method)) this.listeners.set(method, []);
    this.listeners.get(method).push(handler);
  }

  waitForEvent(method, { timeoutMs = STEP_TIMEOUT_MS, sessionId } = {}) {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        remove();
        reject(new Error(`Timeout esperando el evento ${method}`));
      }, timeoutMs);
      const handler = (params, sid) => {
        if (sessionId && sid !== sessionId) return;
        clearTimeout(timer);
        remove();
        resolve(params);
      };
      const remove = () => {
        const handlers = this.listeners.get(method) || [];
        const index = handlers.indexOf(handler);
        if (index >= 0) handlers.splice(index, 1);
      };
      this.on(method, handler);
    });
  }

  close() {
    try {
      this.ws.close();
    } catch {
      // Ignorado: el socket ya puede estar cerrado.
    }
  }
}

// ---------------------------------------------------------------------------
// Arranque de recursos
// ---------------------------------------------------------------------------

/** ¿Indica el log un fallo de bind por puerto ya en uso? */
function isAddrInUse(logText) {
  return /Address already in use|EADDRINUSE|bind\(\) failed/i.test(logText);
}

async function startBackend(tmpDir) {
  if (!existsSync(BIN)) {
    throw new Error(`No existe el binario release ${BIN}. Compílalo con: just build-full`);
  }
  const logPath = path.join(tmpDir, 'backend.log');

  // La reserva del puerto y el `bind` del hijo dejan una ventana sub-milisegundo
  // (TOCTOU) en la que otro proceso puede ocuparlo. Si el arranque falla por
  // `EADDRINUSE`, se reintenta una vez con un puerto nuevo en vez de esperar el
  // timeout completo.
  const maxAttempts = 2;
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const { port, release } = await reservePort();
    const logFd = openSync(logPath, 'a');

    // Libera el puerto reservado justo antes del bind del hijo (minimiza la
    // ventana TOCTOU).
    await release();

    const proc = spawn(BIN, [], {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        BIND_ADDR: `127.0.0.1:${port}`,
        DATABASE_URL: `sqlite://${path.join(tmpDir, 'perf.db')}`,
        OIDC_ENABLED: 'false',
        RUST_LOG: 'warn',
      },
      stdio: ['ignore', logFd, logFd],
    });
    // El hijo ya heredó el descriptor; cerramos el nuestro.
    closeSync(logFd);

    const baseUrl = `http://127.0.0.1:${port}`;
    try {
      await waitFor(
        async () => {
          if (proc.exitCode !== null) throw new Error('el servidor terminó durante el arranque');
          const response = await fetch(`${baseUrl}/`).catch(() => null);
          return response && response.ok;
        },
        { label: 'a que el backend responda GET /' },
      );
      return { proc, baseUrl, logPath };
    } catch (error) {
      const log = readLog(logPath);
      if (attempt < maxAttempts && isAddrInUse(log)) {
        console.warn('  ⚠️  Puerto en uso (EADDRINUSE); reintentando el backend en otro puerto');
        proc.kill('SIGKILL');
        continue;
      }
      throw new Error(`${error.message}\n--- backend.log ---\n${log}`);
    }
  }

  throw new Error('No se pudo arrancar el backend tras reintentar con otro puerto');
}

async function startChrome(tmpDir) {
  const logPath = path.join(tmpDir, 'chrome.log');

  // Igual que el backend: si el `bind` del puerto de DevTools falla por una
  // carrera, se reintenta una vez con otro puerto.
  const maxAttempts = 2;
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const { port, release } = await reservePort();
    const logFd = openSync(logPath, 'a');

    await release();

    const proc = spawn(
      CHROME_BIN,
      [
        '--headless=new',
        `--remote-debugging-port=${port}`,
        '--remote-debugging-address=127.0.0.1',
        '--no-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--no-first-run',
        '--no-default-browser-check',
        '--hide-scrollbars',
        `--user-data-dir=${path.join(tmpDir, 'chrome-profile')}`,
        '--window-size=390,844',
        'about:blank',
      ],
      { stdio: ['ignore', logFd, logFd] },
    );
    closeSync(logFd);

    try {
      const version = await waitFor(
        async () => {
          if (proc.exitCode !== null) throw new Error('Chrome terminó durante el arranque');
          const response = await fetch(`http://127.0.0.1:${port}/json/version`).catch(() => null);
          return response && response.ok ? response.json() : null;
        },
        { label: 'a que Chrome exponga el DevTools Protocol', timeoutMs: 30_000 },
      );
      return { proc, wsUrl: version.webSocketDebuggerUrl, logPath };
    } catch (error) {
      const log = readLog(logPath);
      if (attempt < maxAttempts && isAddrInUse(log)) {
        console.warn('  ⚠️  Puerto en uso (EADDRINUSE); reintentando Chrome en otro puerto');
        proc.kill('SIGKILL');
        continue;
      }
      throw new Error(`${error.message}\n--- chrome.log ---\n${log}`);
    }
  }

  throw new Error('No se pudo arrancar Chrome tras reintentar con otro puerto');
}

// ---------------------------------------------------------------------------
// Navegación y métricas
// ---------------------------------------------------------------------------

async function navigate(cdp, sessionId, url) {
  const loaded = cdp.waitForEvent('Page.loadEventFired', { sessionId });
  await cdp.send('Page.navigate', { url }, sessionId);
  await loaded;
}

async function evaluate(cdp, sessionId, expression, { awaitPromise = false } = {}) {
  const result = await cdp.send(
    'Runtime.evaluate',
    { expression, awaitPromise, returnByValue: true },
    sessionId,
  );
  if (result.exceptionDetails) {
    throw new Error(`evaluación falló: ${result.exceptionDetails.text}`);
  }
  return result.result.value;
}

// FCP: se captura en el instante del paint vía `PerformanceObserver` y se deja
// disponible en `window.__oximealFcp`. El snapshot de recursos se toma aparte,
// DESPUÉS de `Page.loadEventFired` (ver `buildSnapshotExpr`): en el instante del
// FCP los recursos declarados aún en vuelo tendrían `transferSize` 0 y se
// infrafacturarían. El filtro por rutas declaradas ya evita la carrera con los
// chunks asíncronos, así que no hace falta capturar tan pronto.
const FCP_COLLECTOR = `(() => {
  window.__oximealFcp = { ready: false };
  try {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (entry.name === 'first-contentful-paint') {
          window.__oximealFcp = { ready: true, fcpMs: entry.startTime };
          observer.disconnect();
        }
      }
    });
    observer.observe({ type: 'paint', buffered: true });
  } catch {
    // Sin PerformanceObserver no hay medición; se reporta como fallo.
  }
})()`;

// La transferencia inicial se compone de la navegación más el grafo de módulos
// declarado por el `index.html` (entry + `<link rel=modulepreload>` + hojas de
// estilo). Es el equivalente en red del presupuesto de `check-bundle` (el grafo
// estático de entrada): `AppRoot` difiere antd con `React.lazy` precisamente
// para dejarlo fuera de ese grafo (ver su docstring). Las rutas declaradas se
// extraen del HTML servido —no del DOM vivo, que Vite muta inyectando
// `modulepreload` para imports dinámicos— y se fijan aquí. Los chunks
// asíncronos se miden aparte como "diferidos" y no entran en el presupuesto.
const buildSnapshotExpr = (declaredPaths) => `(() => {
  const DECLARED = new Set(${JSON.stringify(declaredPaths)});
  const mapEntry = (entry) => ({
    name: entry.name,
    initiatorType: entry.initiatorType,
    transferSize: entry.transferSize || 0,
  });
  const sum = (entries) => entries.reduce((acc, entry) => acc + (entry.transferSize || 0), 0);
  const isDeclared = (entry) => {
    try { return DECLARED.has(new URL(entry.name).pathname); } catch { return false; }
  };
  const nav = performance.getEntriesByType('navigation')[0] || {};
  const types = ['script', 'link', 'css', 'fetch', 'other'];
  const all = performance.getEntriesByType('resource')
    .filter((entry) => types.includes(entry.initiatorType));
  const initial = all.filter(isDeclared);
  const deferred = all.filter((entry) => !isDeclared(entry));

  const navigationBytes = nav.transferSize || 0;
  const resourcesBytes = sum(initial);
  return {
    navigationBytes,
    resourcesBytes,
    totalBytes: navigationBytes + resourcesBytes,
    deferredBytes: sum(deferred),
    resources: initial.map(mapEntry),
    deferredResources: deferred.map(mapEntry),
  };
})()`;

// ¿Están ya medidos (transferSize > 0) todos los recursos declarados? Se usa
// para esperar, con timeout acotado, a que el grafo inicial tenga su tamaño
// real tras `load`.
const buildSnapshotReadyExpr = (declaredPaths) => `(() => {
  const DECLARED = new Set(${JSON.stringify(declaredPaths)});
  const counted = new Set();
  for (const entry of performance.getEntriesByType('resource')) {
    if (!(entry.transferSize > 0)) continue;
    try {
      const pathname = new URL(entry.name).pathname;
      if (DECLARED.has(pathname)) counted.add(pathname);
    } catch { /* URL inválida */ }
  }
  return counted.size === DECLARED.size;
})()`;

/** Rutas JS/CSS declaradas por el `index.html` (grafo inicial estático). */
async function fetchInitialPaths(baseUrl) {
  const html = await (await fetch(`${baseUrl}/`)).text();
  const paths = new Set();
  const pattern = /(?:src|href)="(\/[^"]+\.(?:js|css))"/g;
  for (const match of html.matchAll(pattern)) paths.add(match[1]);
  return [...paths];
}

const TOUCH_TARGETS_EXPR = `(() => {
  const selectors = [
    { name: 'navegación inferior (botón)', selector: '[data-testid="bottom-nav"] button' },
    { name: 'navegación inferior (enlace asistente)', selector: '[data-testid="bottom-nav"] a' },
    { name: 'alta de compra (Añadir)', selector: '.add-item-form__submit' },
  ];
  const targets = [];
  for (const { name, selector } of selectors) {
    for (const element of Array.from(document.querySelectorAll(selector))) {
      const rect = element.getBoundingClientRect();
      targets.push({
        name,
        label: (element.textContent || '').trim().replace(/\\s+/g, ' ').slice(0, 40),
        width: Math.round(rect.width * 100) / 100,
        height: Math.round(rect.height * 100) / 100,
      });
    }
  }
  return targets;
})()`;

// ---------------------------------------------------------------------------
// Programa principal
// ---------------------------------------------------------------------------

const tmpDir = mkdtempSync(path.join(tmpdir(), 'oximeal-perf-web-'));
const cleanupFns = [];

function cleanup() {
  for (const fn of cleanupFns.reverse()) {
    try {
      fn();
    } catch {
      // Ignorado durante la limpieza.
    }
  }
  rmSync(tmpDir, { recursive: true, force: true });
}

const watchdog = setTimeout(() => {
  console.error('\n❌ Timeout global de la auditoría web');
  cleanup();
  process.exit(1);
}, GLOBAL_TIMEOUT_MS);
watchdog.unref();

try {
  let baseUrl = process.env.APP_URL;
  if (baseUrl) {
    console.log(`▶ Auditando servidor existente en ${baseUrl}`);
  } else {
    console.log('▶ Arrancando el binario release con la UI embebida');
    const backend = await startBackend(tmpDir);
    baseUrl = backend.baseUrl;
    cleanupFns.push(() => backend.proc.kill('SIGKILL'));
    console.log(`▶ Backend en ${baseUrl} (pid ${backend.proc.pid})`);
  }

  const chrome = await startChrome(tmpDir);
  cleanupFns.push(() => chrome.proc.kill('SIGKILL'));

  const initialPaths = await fetchInitialPaths(baseUrl);
  // Guard anti-falso-negativo: si el `index.html` no declarara rutas JS/CSS (p.
  // ej. porque cambió el formato del build y el regex dejó de casar), el
  // presupuesto de transferencia pasaría contando solo la navegación (~1,6 kB)
  // y no detectaría regresiones. Es un fallo, no un resultado válido.
  if (initialPaths.length === 0) {
    throw new Error(
      `No se extrajo ninguna ruta JS/CSS del index.html de ${baseUrl}. ` +
        'El regex del grafo inicial declarado podría no casar con el build actual; ' +
        'revisa scripts/perf-web.mjs (fetchInitialPaths). Medir solo la navegación ' +
        'daría un falso positivo del presupuesto.',
    );
  }
  console.log(`▶ Grafo inicial declarado por index.html: ${initialPaths.length} recursos`);

  const browser = await Cdp.connect(chrome.wsUrl);
  cleanupFns.push(() => browser.close());

  const { targetId } = await browser.send('Target.createTarget', { url: 'about:blank' });
  const { sessionId } = await browser.send('Target.attachToTarget', { targetId, flatten: true });

  await browser.send('Page.enable', {}, sessionId);
  await browser.send('Runtime.enable', {}, sessionId);
  await browser.send(
    'Emulation.setDeviceMetricsOverride',
    { width: 390, height: 844, deviceScaleFactor: 1, mobile: true },
    sessionId,
  );
  await browser.send(
    'Page.addScriptToEvaluateOnNewDocument',
    { source: FCP_COLLECTOR },
    sessionId,
  );
  cleanupFns.push(() =>
    browser.send('Emulation.clearDeviceMetricsOverride', {}, sessionId).catch(() => {}),
  );

  // --- Navegación inicial al menú, con métricas de arranque ---

  console.log(`▶ Navegando a ${baseUrl}/ (viewport móvil 390×844)`);
  await navigate(browser, sessionId, `${baseUrl}/`);

  // FCP del paint; el snapshot de transferencia se toma ya con `load` cumplido.
  const fcpMs = await waitFor(
    () =>
      evaluate(
        browser,
        sessionId,
        'window.__oximealFcp && window.__oximealFcp.ready ? window.__oximealFcp.fcpMs : null',
      ),
    { label: 'a que el navegador registre el FCP' },
  );
  // Espera acotada a que todos los recursos declarados tengan `transferSize`.
  await waitFor(
    () => evaluate(browser, sessionId, buildSnapshotReadyExpr(initialPaths)),
    { label: 'a que el grafo inicial tenga tamaño de transferencia', timeoutMs: 5000 },
  ).catch(() => {
    console.warn('  ⚠️  Algún recurso declarado no registró transferSize; se mide lo disponible');
  });
  const snapshot = await evaluate(browser, sessionId, buildSnapshotExpr(initialPaths));
  const metrics = { fcpMs, ...snapshot };
  const navTargets = await evaluate(browser, sessionId, TOUCH_TARGETS_EXPR);

  // --- Navegación al modo compra para el botón de alta ---

  console.log(`▶ Navegando a ${baseUrl}/shopping`);
  await navigate(browser, sessionId, `${baseUrl}/shopping`);
  await waitFor(
    () => evaluate(browser, sessionId, `!!document.querySelector('.add-item-form__submit')`),
    { label: 'al formulario de alta de la compra' },
  );
  const shoppingTargets = await evaluate(browser, sessionId, TOUCH_TARGETS_EXPR);

  // --- Informe ---

  const transferKb = metrics.totalBytes / 1024;

  console.log('\n──────────────────────────────────────────────');
  console.log('Rendimiento web');
  console.log(
    `  FCP.................... ${fcpMs == null ? 'n/d' : `${fcpMs.toFixed(0)} ms`} (presupuesto ≤${FCP_BUDGET_MS} ms)`,
  );
  console.log(
    `  Transferencia declarada.. ${transferKb.toFixed(1)} kB (presupuesto ≤${(TRANSFER_BUDGET_BYTES / 1024).toFixed(0)} kB)`,
  );
  console.log(`    ↳ navegación......... ${(metrics.navigationBytes / 1024).toFixed(1)} kB`);
  console.log(`    ↳ recursos............ ${(metrics.resourcesBytes / 1024).toFixed(1)} kB (${metrics.resources.length} recursos)`);
  for (const resource of metrics.resources.sort((a, b) => b.transferSize - a.transferSize)) {
    const name = resource.name.replace(baseUrl, '');
    console.log(
      `        ${(resource.transferSize / 1024).toFixed(1).padStart(8)} kB  [${resource.initiatorType}] ${name}`,
    );
  }
  if (metrics.deferredResources.length > 0) {
    console.log(
      `    ↳ chunks diferidos*... ${(metrics.deferredBytes / 1024).toFixed(1)} kB (${metrics.deferredResources.length} recursos, fuera del presupuesto)`,
    );
    for (const resource of metrics.deferredResources.sort((a, b) => b.transferSize - a.transferSize)) {
      const name = resource.name.replace(baseUrl, '');
      console.log(
        `        ${(resource.transferSize / 1024).toFixed(1).padStart(8)} kB  [${resource.initiatorType}] ${name}`,
      );
    }
    console.log(
      '      * imports dinámicos en runtime (p. ej. antd vía React.lazy): no forman parte del grafo inicial.',
    );
  }

  console.log('\nObjetivos táctiles (mínimo 48×48 px)');
  const allTargets = [...navTargets, ...shoppingTargets];
  if (allTargets.length === 0) {
    console.log('  (no se encontró ningún control principal)');
  }
  for (const target of allTargets) {
    const ok = target.width >= TOUCH_MIN_PX && target.height >= TOUCH_MIN_PX;
    console.log(
      `  ${ok ? '✅' : '❌'} ${target.width.toFixed(0)}×${target.height.toFixed(0)} px  ${target.name}: "${target.label}"`,
    );
  }
  const assistantTargets = allTargets.filter((t) => t.name.includes('asistente'));
  if (assistantTargets.length === 0) {
    console.log('  ℹ️  sin enlace de asistente (VALET_URL no configurada): omitido');
  }
  console.log('──────────────────────────────────────────────');

  // --- Veredicto ---

  const failures = [];
  if (fcpMs == null) failures.push('no se registró FCP');
  else if (fcpMs > FCP_BUDGET_MS) failures.push(`FCP ${fcpMs.toFixed(0)} ms > ${FCP_BUDGET_MS} ms`);
  if (metrics.totalBytes > TRANSFER_BUDGET_BYTES) {
    failures.push(`transferencia ${transferKb.toFixed(1)} kB > ${(TRANSFER_BUDGET_BYTES / 1024).toFixed(0)} kB`);
  }
  for (const target of allTargets) {
    if (target.width < TOUCH_MIN_PX || target.height < TOUCH_MIN_PX) {
      failures.push(`objetivo táctil <${TOUCH_MIN_PX}px: ${target.name} ${target.width}×${target.height}`);
    }
  }

  if (failures.length > 0) {
    console.error('\n❌ Presupuestos web incumplidos:');
    for (const failure of failures) console.error(`   - ${failure}`);
    cleanup();
    process.exit(1);
  }

  console.log('\n✅ Presupuestos web cumplidos (FCP, transferencia y objetivos táctiles)');
  cleanup();
} catch (error) {
  console.error(`\n❌ Auditoría web fallida: ${error.message}`);
  cleanup();
  process.exit(1);
}
