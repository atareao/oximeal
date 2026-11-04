# Tasks — Notas de release desde el CHANGELOG (hotfix de la Fase 7)

## 1. Preparación
- [x] 1.1 Leer `.github/workflows/release.yml`, `cliff.toml` y `openspec/specs/release/spec.md`
- [x] 1.2 Diagnosticar el fallo del job `release` del run `41783524262` (descarga de todos los artefactos, incluido el caché de Docker)
- [x] 1.3 Diagnosticar el segundo fallo (`git-cliff-action@v4` no admite `output` → `RELEASE_NOTES.md` inexistente)
- [x] 1.4 Diagnosticar la causa estructural (squash-merge en `main` → `git-cliff --current` ve 1 solo commit)

## 2. Corrección
- [x] 2.1 `upload-artifact` con nombre explícito `oximeal-${{ matrix.target }}`
- [x] 2.2 `download-artifact` con `pattern: oximeal-*` (excluye el caché de Docker)
- [x] 2.3 Script `.github/scripts/release-notes.sh` que extrae la sección del CHANGELOG y omite subsecciones vacías
- [x] 2.4 Job `release` usa el script; `git-cliff` queda como fallback no fatal

## 3. Verificación
- [x] 3.1 `bash -n` y prueba local del script con `v0.5.0` (sin subsecciones vacías) y con una versión inexistente (fallback)
- [x] 3.2 `actionlint` sobre `release.yml`
- [x] 3.3 Reejecutar la release (`workflow_dispatch tag=v0.5.0`): los 8 jobs en verde, incluido `GitHub Release`
- [x] 3.4 Verificar la release `v0.5.0`: assets `.tar.gz` + `.sha256` (sha256 válidos), imagen `ghcr.io/atareao/oximeal:0.5.0` y `:latest`, y notas completas

## 4. Cierre
- [x] 4.1 Actualizar el `CHANGELOG.md` y `PLAN.md` si procede
- [x] 4.2 Archivar el change (`openspec archive`) y validar `--all --strict`
- [x] 4.3 Sincronizar `main` → `development`
