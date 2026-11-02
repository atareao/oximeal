#!/usr/bin/env node
/**
 * Compone `dist/sw.js` a partir de la plantilla que Vite copia desde
 * `public/sw.js`:
 *
 * - inyecta la lista de assets con hash del build (precache real del shell),
 * - inyecta una versión de caché derivada de ese listado, de modo que cada
 *   build con assets distintos invalide las cachés anteriores sin bumps manuales.
 */
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';

const DIST = new URL('../dist/', import.meta.url);
const MANIFEST = new URL('.vite/manifest.json', DIST);
const SW = new URL('sw.js', DIST);

let manifest;
try {
  manifest = JSON.parse(await readFile(MANIFEST, 'utf8'));
} catch (error) {
  throw new Error(
    `build-sw: no se pudo leer ${MANIFEST.pathname}; ejecuta \`vite build\` antes. Causa: ${error}`,
  );
}

const assets = new Set();
for (const entry of Object.values(manifest)) {
  if (entry.file) {
    assets.add(`/${entry.file}`);
  }
  for (const css of entry.css ?? []) {
    assets.add(`/${css}`);
  }
  for (const asset of entry.assets ?? []) {
    assets.add(`/${asset}`);
  }
}

const precache = [...assets].sort();
const version = createHash('sha256').update(precache.join('\n')).digest('hex').slice(0, 12);

if (precache.length === 0) {
  throw new Error('build-sw: el manifest de Vite no contiene assets; el precache quedaría vacío');
}

const missing = precache.filter((asset) => !existsSync(new URL(`.${asset}`, DIST)));
if (missing.length > 0) {
  throw new Error(`build-sw: assets del manifest ausentes en dist: ${missing.join(', ')}`);
}

let source = await readFile(SW, 'utf8');

const VERSION_TOKEN = "'__CACHE_VERSION__'";
const PRECACHE_TOKEN = 'const PRECACHE_ASSETS = [];';
if (!source.includes(VERSION_TOKEN) || !source.includes(PRECACHE_TOKEN)) {
  throw new Error(
    'build-sw: no se encontraron los marcadores de inyección en dist/sw.js; revisa public/sw.js',
  );
}

source = source.replace(VERSION_TOKEN, `'${version}'`);
source = source.replace(
  PRECACHE_TOKEN,
  `const PRECACHE_ASSETS = ${JSON.stringify(precache, null, 2)};`,
);
await writeFile(SW, source);

console.log(`sw.js: ${precache.length} assets precacheados (versión ${version})`);
