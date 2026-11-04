# Tasks — Dependabot

## 1. Configuración
- [x] 1.1 Crear `.github/dependabot.yml` (cargo, npm/pnpm, github-actions, docker)
- [x] 1.2 `target-branch: development` en las cuatro entradas (gitflow)
- [x] 1.3 Grupos `minor`+`patch` por ecosistema; `major` en PRs separadas
- [x] 1.4 Validar la sintaxis YAML del fichero

## 2. Verificación
- [x] 2.1 `openspec validate --all --strict`
- [x] 2.2 Primera ejecución de Dependabot: PRs hacia `development` (tras mergear a `main`) — 7 PRs abiertas (#73–#79), todas con base `development`

## 3. Cierre
- [x] 3.1 Entrada en `CHANGELOG.md` (`[No publicado]`)
- [x] 3.2 Archivar el change
