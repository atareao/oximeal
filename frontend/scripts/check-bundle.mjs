// Presupuesto del grafo estático inicial.
//
// Lee el manifest de Vite, localiza el chunk de entrada (`isEntry: true`) y
// recorre recursivamente sus `imports` estáticos sumando el tamaño real en
// disco de cada fichero emitido. Si el total supera el presupuesto, sale con
// código 1.
//
// ESM sin dependencias: solo `node:fs`, `node:path` e `import.meta.url`.

import { readFileSync, statSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const BUDGET_BYTES = 500 * 1024;

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function findManifest() {
  const candidates = [
    path.join(rootDir, 'dist', '.vite', 'manifest.json'),
    path.join(rootDir, 'dist', 'manifest.json'),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  throw new Error(`No se encontró el manifest de Vite. Buscado en:\n  ${candidates.join('\n  ')}`);
}

function fileSize(relativeFile) {
  const absolute = path.join(rootDir, 'dist', relativeFile);
  return statSync(absolute).size;
}

function main() {
  const manifestPath = findManifest();
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));

  const entryKey = Object.keys(manifest).find((key) => manifest[key].isEntry);
  if (!entryKey) {
    throw new Error('El manifest no contiene ningún chunk con "isEntry: true"');
  }

  const visited = new Set();
  let totalBytes = 0;
  const included = [];

  const walk = (key) => {
    if (visited.has(key)) {
      return;
    }
    visited.add(key);

    const chunk = manifest[key];
    if (!chunk) {
      return;
    }

    if (chunk.file) {
      const size = fileSize(chunk.file);
      totalBytes += size;
      included.push({ file: chunk.file, size });
    }

    for (const imported of chunk.imports ?? []) {
      walk(imported);
    }
  };

  walk(entryKey);

  const totalKb = totalBytes / 1024;
  const budgetKb = BUDGET_BYTES / 1024;

  console.log('Presupuesto del grafo estático inicial:');
  for (const { file, size } of included.sort((a, b) => b.size - a.size)) {
    console.log(`  ${(size / 1024).toFixed(1).padStart(8)} kB  ${file}`);
  }
  console.log(`  ${'-'.repeat(8)}`);
  console.log(
    `  ${totalKb.toFixed(1).padStart(8)} kB  total (${included.length} chunks, presupuesto ${budgetKb} kB)`,
  );

  if (totalBytes > BUDGET_BYTES) {
    console.error(
      `\n❌ El grafo estático inicial supera el presupuesto: ${totalKb.toFixed(1)} kB > ${budgetKb} kB`,
    );
    process.exit(1);
  }

  console.log(`\n✅ Grafo estático inicial dentro del presupuesto: ${totalKb.toFixed(1)} kB`);
}

main();
