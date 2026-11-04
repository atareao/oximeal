# Change: mantenimiento del CI (acciones, coherencia de versión y puerta de scripts)

## Why

Tres deudas de mantenimiento detectadas al cerrar la Fase 7:

1. Las acciones de los workflows usan mayores con Node 20, que GitHub ya ejecuta
   forzosamente sobre Node 24 y marca como *deprecated* en cada run
   (`actions/checkout@v4`, `setup-node@v4`, `upload-artifact@v4`,
   `download-artifact@v4`, `pnpm/action-setup@v4`, `attest-build-provenance@v2`,
   `docker/*`). Además `ubuntu-latest` avisa de la migración a Ubuntu 26.
2. `check-version.sh` **no** comprobaba la versión explícita de
   `backend/Cargo.toml`, que **no** hereda de `[workspace.package]`. En el *bump*
   a 0.5.0 hubo que corregirla a mano y el script no habría avisado del olvido.
3. Los scripts de `.github/scripts/` no tenían ninguna prueba ni revisión
   estática: los errores de `release.yml` (#64, #65, #66) se detectaron en el
   propio run de release, no antes.

## What Changes

- `check-version.sh`: comprueba también `backend/Cargo.toml` y lo incluye en la
  traza de diagnóstico.
- Nuevo `.github/scripts/test-gh-scripts.sh`: 14 casos que cubren
  `check-version.sh` (coherencia de las cuatro fuentes, tag no SemVer) y
  `release-notes.sh` (encabezado de versión, entradas, omisión de subsecciones
  vacías, no arrastrar la versión anterior y *fallback* sin sección).
- Nueva puerta `Scripts (shell)` en `ci.yml` (`shellcheck` + los tests), que
  hereda el job `quality` de `release.yml`.
- Acciones a las mayores actuales (Node 24) y runners fijados a `ubuntu-24.04`
  (incluido el `os` de la matriz), evitando depender de `ubuntu-latest`.

## Non-goals

- No se cambia ninguna funcionalidad del producto (ni API, ni UI, ni esquema).
- No se introducen *pins* por SHA (el repo usa etiquetas de mayor); sería otra
  decisión de política de CI.

## Impact

- Capability `release`: MODIFIED «Puertas de calidad antes de publicar» y
  «Coherencia de versionado (ADR-33)».
- Ficheros: `.github/workflows/ci.yml`, `.github/workflows/release.yml`,
  `.github/scripts/{check-version.sh,release-notes.sh,test-gh-scripts.sh}`.
