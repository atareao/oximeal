# Change: la imagen nunca publica el stub de caché

## Why

La imagen `ghcr.io/atareao/oximeal:0.6.0` publicada está **rota**: el
`/usr/local/bin/oximeal-backend` que contiene pesa **285 KB** (un `fn main() {}`)
en vez de los 16,8 MB del binario real. El contenedor arranca y termina con
código 0 sin escribir una sola línea de log, así que `podman run` parece
correcto pero no sirve ninguna petición.

Causa: el stage `builder` compila a propósito un stub (`fn main() {}`) para
cachear las dependencias de Cargo y, acto seguido, copia el código real y
recompila. BuildKit conserva los mtimes originales del contexto, que son
**anteriores** a los artefactos generados por el stub; Cargo compara mtimes y
concluye que el crate está fresco, así que **no recompila** y el stage `runtime`
copia el stub. El build termina con código 0 y el fallo es silencioso.

Los `.tar.gz` de la release no están afectados: solo la imagen.

## What Changes

- **Invalidar explícitamente el artefacto del stub** antes del build real:
  `cargo clean -p oximeal-backend` (o borrado de `target/release/oximeal-backend`
  y sus fingerprints) más `touch` de los fuentes copiados, de modo que Cargo
  recompile siempre el crate propio sin perder la caché de dependencias.
- **Guardia de verificación en el propio Dockerfile**: tras el build real,
  comprobar que el binario existe y supera un tamaño mínimo (p. ej. `test -s` y
  `stat`), para que un fallo equivalente rompa el build en vez de publicar una
  imagen silenciosamente inservible.
- Sin cambios de runtime, de variables de entorno, de volumen ni de compose.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `deployment`: el requirement «Imagen multi-stage reproducible» añade el
  escenario de que el binario del stage `runtime` es el compilado con el código
  real y nunca el stub de calentamiento de caché, y que el build falla si el
  binario final no supera la verificación de tamaño.

## Impact

- `Dockerfile` (stage `builder`).
- Spec: `openspec/specs/deployment/spec.md` (MODIFIED del requirement citado).
- Sin cambios en `backend/`, `frontend/`, `deploy/compose.yaml` ni migraciones.
- Release: requiere publicar una versión nueva (patch) para regenerar la imagen.
