# Capability: release

## MODIFIED Requirements

### Requirement: GitHub Release con notas generadas

El workflow SHALL crear la **GitHub Release** del tag adjuntando los artefactos
binarios y con **notas de versión** tomadas de la sección curada del
`CHANGELOG.md` para esa versión (la misma que verifica `check-version.sh`,
ADR-33), a través del script versionado `.github/scripts/release-notes.sh`. El
recurso descriptivo SHALL ser el `CHANGELOG.md`, no la reconstrucción de los
commits del rango: `main` integra por *squash-merge* y el rango entre tags
contiene un único commit, por lo que `git-cliff --current` no puede reproducir
las notas. `git-cliff` (`cliff.toml`) MAY usarse como *fallback* solo si el
CHANGELOG no contuviera la sección de la versión; en tal caso, y si tampoco
`git-cliff` pudiera generar nada, la release SHALL crearse igualmente con una
nota mínima que incluya el tag. Las notas SHALL ser coherentes con las secciones
de `CHANGELOG.md` y omitir los encabezados de subsección sin contenido.

El job `release` SHALL descargar **solo** los artefactos binarios del run
(`pattern: oximeal-*`), de modo que la presencia de un artefacto de caché de
Docker Buildx (`*.dockerbuild`) no rompa la descarga.

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

#### Scenario: Descarga de artefactos acotada a los binarios

- GIVEN un run de release que publica artefactos binarios `oximeal-<target>` y un artefacto de caché de Docker Buildx (`*.dockerbuild`)
- WHEN el job de release descarga los artefactos con `pattern: oximeal-*`
- THEN descarga solo los `.tar.gz` y sus `.sha256`
- AND la presencia del artefacto de caché de Docker no rompe la descarga

#### Scenario: Sin sección en el CHANGELOG

- GIVEN un tag cuya versión no tiene sección en `CHANGELOG.md`
- WHEN el job de release genera las notas
- THEN se intenta `git-cliff` como fallback
- AND la release se crea igualmente sin fallar, con al menos una nota mínima que incluya el tag
