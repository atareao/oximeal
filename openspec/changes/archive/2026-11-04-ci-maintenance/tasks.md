# Tasks — Mantenimiento del CI

## 1. Acciones y runners
- [x] 1.1 Inventariar las acciones y sus mayores actuales (GitHub API)
- [x] 1.2 Verificar que los inputs usados siguen existiendo (`upload-artifact@v7`, `download-artifact@v8`, `setup-node@v7`, `pnpm/action-setup@v6`, `attest-build-provenance@v4`, `build-push-action@v7`)
- [x] 1.3 Subir las acciones a las mayores actuales (Node 24) en `ci.yml` y `release.yml`
- [x] 1.4 Fijar los runners a `ubuntu-24.04` (incluido el `os` de la matriz) para no depender de `ubuntu-latest`

## 2. Coherencia de versión
- [x] 2.1 `check-version.sh` valida también la `version` explícita de `backend/Cargo.toml`
- [x] 2.2 Añadir la lectura y el mensaje de error de `backend/Cargo.toml`

## 3. Puerta de scripts
- [x] 3.1 Nuevo `.github/scripts/test-gh-scripts.sh` (14 casos: coherencia de versiones y notas de release)
- [x] 3.2 Job `Scripts (shell)` en `ci.yml` con `shellcheck` + los tests
- [x] 3.3 `actionlint` sobre ambos workflows

## 4. Verificación
- [x] 4.1 `bash .github/scripts/test-gh-scripts.sh` en local: 14/14
- [x] 4.2 `check-version.sh v0.5.0` sobre el repo real: coherente (incluye backend)
- [x] 4.3 `openspec validate --all --strict`
- [ ] 4.4 `workflow_dispatch tag=v0.5.0`: pipeline completo en verde con las acciones nuevas (tras mergear)

## 5. Cierre
- [x] 5.1 Entrada en `CHANGELOG.md` (`[No publicado]`)
- [x] 5.2 Archivar el change (la sincronización `main` → `development` se hace tras el merge)
