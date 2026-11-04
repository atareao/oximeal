# Tasks — Generador de notas desde el ref de ejecución

## 1. Diagnóstico
- [x] 1.1 Reproducir el fallo (`workflow_dispatch tag=v0.5.0`, run 41789091012): `exit 127` en «Generar notas de la release»
- [x] 1.2 Confirmar que `.github/scripts/release-notes.sh` no existe en el árbol del tag `v0.5.0`

## 2. Cambio
- [x] 2.1 El job `release` hace checkout de `github.sha` (no del tag)
- [x] 2.2 `actionlint` limpio en `release.yml`
- [x] 2.3 Alinear el `Purpose` de la capability `release`

## 3. Verificación
- [x] 3.1 `openspec validate --all --strict`
- [ ] 3.2 `workflow_dispatch tag=v0.5.0`: pipeline 8/8 en verde y notas regeneradas desde el CHANGELOG (tras mergear)

## 4. Cierre
- [x] 4.1 Entrada en `CHANGELOG.md` (`[No publicado]`)
- [x] 4.2 Archivar el change (la sincronización `main` → `development` se hace tras el merge)
