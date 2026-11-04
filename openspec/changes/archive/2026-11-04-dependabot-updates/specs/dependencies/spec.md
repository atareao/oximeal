# Capability: dependencies

## ADDED Requirements

### Requirement: Actualización automatizada de dependencias

El repositorio SHALL incluir `.github/dependabot.yml` (version 2) con
actualizaciones semanales para `cargo` (raíz del workspace), `npm`
(`/frontend`, pnpm), `github-actions` y `docker`. Cada entrada SHALL fijar
`target-branch: development`, agrupar `minor`+`patch` y limitar las PRs abiertas;
las `major` SHALL ir en PRs separadas.

#### Scenario: Las actualizaciones llegan a development, nunca a main

- GIVEN el fichero `.github/dependabot.yml` presente en la rama por defecto
- WHEN Dependabot abre una PR de actualización de dependencias
- THEN la PR tiene `development` como rama base
- AND no se abre ninguna PR de dependencias directamente contra `main`

#### Scenario: Agrupación de minor y patch

- GIVEN varias dependencias de un mismo ecosistema con versiones minor o patch nuevas
- WHEN se ejecuta la programación semanal de Dependabot
- THEN Dependabot abre un único PR agrupado para ese ecosistema

#### Scenario: Las versiones major no se agrupan

- GIVEN una dependencia con una versión major nueva
- WHEN se ejecuta la programación semanal de Dependabot
- THEN Dependabot abre un PR independiente para esa actualización
