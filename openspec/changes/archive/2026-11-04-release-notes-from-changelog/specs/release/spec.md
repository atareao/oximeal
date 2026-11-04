# Capability: release

## MODIFIED Requirements

### Requirement: GitHub Release con notas generadas

El workflow SHALL crear la **GitHub Release** del tag adjuntando los artefactos
binarios y con **notas** tomadas de la sección curada de `CHANGELOG.md` para esa
versión, mediante el script `.github/scripts/release-notes.sh`. El `CHANGELOG.md`
es el recurso descriptivo: `main` integra por *squash-merge* y el rango entre
tags contiene un único commit, así que `git-cliff --current` no reproduce las
notas; `git-cliff` MAY usarse solo como *fallback* no fatal.

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

## ADDED Requirements

### Requirement: Descarga acotada de artefactos en el job de release

El job `release` SHALL descargar **solo** los artefactos binarios del run
(`pattern: oximeal-*`), para que un artefacto de caché de Docker Buildx
(`*.dockerbuild`) no rompa la descarga.

#### Scenario: Descarga acotada a los binarios

- GIVEN un run de release que publica artefactos binarios `oximeal-<target>` y un artefacto de caché de Docker Buildx (`*.dockerbuild`)
- WHEN el job de release descarga los artefactos con `pattern: oximeal-*`
- THEN descarga solo los `.tar.gz` y sus `.sha256`
- AND la presencia del artefacto de caché de Docker no rompe la descarga
