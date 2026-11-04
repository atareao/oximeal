# Capability: release

## MODIFIED Requirements

### Requirement: GitHub Release con notas generadas

El workflow SHALL crear la GitHub Release del tag adjuntando los artefactos
binarios y con notas de la sección curada de `CHANGELOG.md` para esa versión,
mediante `.github/scripts/release-notes.sh`. El job de release SHALL tomar el
generador y el `CHANGELOG.md` del ref que dispara la ejecución (`github.sha`), no
del tag, que puede ser anterior a esos ficheros. `git-cliff` MAY usarse solo como
*fallback* no fatal.

#### Scenario: Release creada con binarios y notas

- GIVEN un tag `v0.5.0`, los artefactos y la imagen publicados y una sección `## [0.5.0]` en `CHANGELOG.md`
- WHEN se ejecuta el job de release
- THEN se crea una GitHub Release para `v0.5.0`
- AND la release adjunta los `.tar.gz` y sus `.sha256`
- AND el cuerpo de la release contiene las entradas de la sección `## [0.5.0]` del CHANGELOG
- AND no aparecen encabezados de subsección vacíos (p. ej. un `### Corregido` sin entradas)

#### Scenario: Sin commits publicables

- GIVEN un rango de commits que no contiene entradas de usuario relevantes (solo `chore`/`ci`)
- WHEN el job de release genera las notas
- THEN la release se crea igualmente
- AND el cuerpo de la release no queda vacío de forma silenciosa (incluye al menos la sección del CHANGELOG para el tag)

#### Scenario: Sin sección en el CHANGELOG

- GIVEN un tag cuya versión no tiene sección en `CHANGELOG.md`
- WHEN el job de release genera las notas
- THEN se intenta `git-cliff` como fallback
- AND la release se crea igualmente sin fallar, con al menos una nota mínima que incluya el tag

#### Scenario: El generador de notas proviene del ref de ejecución

- GIVEN un `workflow_dispatch` con `tag = v0.5.0` cuyo árbol en el tag no contiene `.github/scripts/release-notes.sh`
- WHEN se ejecuta el job de release
- THEN el checkout del job de release usa el commit que disparó la ejecución, no el tag
- AND las notas se generan con el `CHANGELOG.md` de ese commit
- AND la release se crea o actualiza sin fallar
