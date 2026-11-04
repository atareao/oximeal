# release Specification

## Purpose

Esta capability define la **automatización de releases** de OxiMeal: un workflow
`.github/workflows/release.yml` disparado por tags `v<version>` (o por
`workflow_dispatch`) que, tras reutilizar las puertas de calidad de `ci.yml` vía
`workflow_call`, publica artefactos binarios `x86_64`/`aarch64` (`.tar.gz` +
`.sha256`), una imagen en GHCR (`<version>` y `latest`, sin retroceder con
pre-releases) y una GitHub Release con notas tomadas de la sección curada de
`CHANGELOG.md` para esa versión (con `git-cliff` solo como *fallback*), con
provenance de build. Incluye la verificación de coherencia entre el tag, `[workspace.package]
version`, `frontend/package.json` y `CHANGELOG.md` (ADR-33).

## Requirements

### Requirement: Disparo del pipeline de release por tag

El sistema SHALL incluir un workflow `.github/workflows/release.yml` disparado por
la creación de **tags que empiecen por `v`** (`push: tags: ['v*']`) y, de forma
opcional, por `workflow_dispatch` (input `tag`). El workflow SHALL ejecutarse
**únicamente** sobre tags (o sobre el tag indicado a mano) y **nunca** sobre
`pull_request`.

#### Scenario: Publicación disparada por un tag de versión

- GIVEN un tag `v0.5.0` creado sobre `main`
- WHEN se hace `push` del tag
- THEN se ejecuta el workflow `release.yml`
- AND el workflow arranca el job de publicación con la versión derivada del tag

#### Scenario: Un pull request no publica

- GIVEN un pull request abierto hacia `main` o `development`
- WHEN se ejecutan los workflows del repositorio
- THEN solo se ejecuta `ci.yml`
- AND `release.yml` no se ejecuta y no se publica ningún artefacto, imagen ni release

#### Scenario: Ejecución manual con un tag concreto

- GIVEN el workflow `release.yml` en la rama por defecto
- WHEN se lanza `workflow_dispatch` con el input `tag = v0.5.0`
- THEN el workflow opera sobre ese tag igual que si se hubiera creado por `push`

### Requirement: Puertas de calidad antes de publicar

El workflow SHALL ejecutar, antes de los jobs de publicación, las mismas puertas
de calidad que `ci.yml`: backend (`fmt --check`, `clippy -D warnings`, `test` con y
sin `embed-ui`), frontend (`typecheck`, `format:check`, `test`, `lint`, `build`,
`check-bundle`) y scripts (`shellcheck` + tests de `.github/scripts`). Si cualquier
puerta falla, SHALL terminar sin publicar artefactos, imagen ni release.

#### Scenario: Puertas en verde habilitan la publicación

- GIVEN un tag de versión válido
- WHEN se ejecuta el workflow de release
- THEN se ejecutan las puertas de backend, frontend y scripts
- AND, si todas pasan, continúan los jobs de artefactos, imagen y release

#### Scenario: Fallo de build impide publicar

- GIVEN un tag de versión válido y un fallo en `cargo test`, `clippy` o el build del frontend
- WHEN se ejecuta el workflow de release
- THEN el workflow falla antes de cualquier paso de publicación
- AND no se crea ni artefacto binario, ni imagen en GHCR, ni GitHub Release

#### Scenario: La puerta de scripts valida los scripts de CI

- GIVEN un script de `.github/scripts/` con un aviso de `shellcheck` o un test que falla
- WHEN se ejecuta la puerta de scripts
- THEN la puerta falla y el workflow no publica nada

### Requirement: Artefactos binarios por arquitectura

El workflow SHALL compilar el binario `oximeal-backend` con `--release --features
embed-ui` para **linux `x86_64-unknown-linux-gnu`** y **linux
`aarch64-unknown-linux-gnu`** (matriz de runners nativos de ARM o `cross`, según
la decisión del ADR-33), empaquetarlo como
`oximeal-<version>-<target>.tar.gz`, calcular su `sha256` y adjuntarlos a la
GitHub Release.

#### Scenario: Artefactos generados para ambas arquitecturas

- GIVEN un tag `v0.5.0` y las puertas de calidad en verde
- WHEN se ejecutan los jobs de build de la matriz
- THEN se producen `oximeal-0.5.0-x86_64-unknown-linux-gnu.tar.gz` y
  `oximeal-0.5.0-aarch64-unknown-linux-gnu.tar.gz`
- AND cada archivo tiene su `.sha256` y queda adjunto a la release del tag

### Requirement: Publicación de la imagen en GHCR

El workflow SHALL publicar la imagen del change E1 `container-image` en GitHub
Container Registry como `ghcr.io/<owner>/oximeal`, con una etiqueta **semver**
derivada del tag (`<version>`, p. ej. `0.5.0`) y la etiqueta `latest`,
autenticándose con el `GITHUB_TOKEN` y el permiso `packages: write`.

#### Scenario: Imagen publicada con semver y latest

- GIVEN un tag `v0.5.0` y las puertas de calidad en verde
- WHEN se ejecuta el job de imagen
- THEN `ghcr.io/<owner>/oximeal:0.5.0` y `ghcr.io/<owner>/oximeal:latest` apuntan a la imagen construida para ese tag

#### Scenario: El tag `latest` solo avanza en releases

- GIVEN una release publicada desde el tag más reciente
- WHEN se crea un tag anterior
- THEN la etiqueta `latest` permanece en la release más reciente y no retrocede

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

### Requirement: Coherencia de versionado (ADR-33)

La versión del proyecto SHALL tener como fuente de verdad `[workspace.package]
version` de `Cargo.toml`. El workflow SHALL verificar que el tag,
`[workspace.package] version`, la `version` explícita de `backend/Cargo.toml`, la
`version` de `frontend/package.json` y la sección más reciente de
`CHANGELOG.md` son coherentes; si no lo son, SHALL fallar antes de publicar. El
*bump* SHALL ser manual en un PR de release a `main` y el tag SHALL tener el
formato `v<version>` (SemVer).

#### Scenario: Tag y versión coherentes

- GIVEN `Cargo.toml` con `version = "0.5.0"`, `backend/Cargo.toml` con `version = "0.5.0"`, `frontend/package.json` con `"version": "0.5.0"` y el tag `v0.5.0`
- WHEN se ejecuta el workflow de release
- THEN la comprobación de coherencia pasa
- AND el workflow continúa con la publicación

#### Scenario: Tag inválido o incoherente

- GIVEN un tag que no empieza por `v`, con formato no SemVer, o cuyo número no coincide con la `version` del workspace
- WHEN se ejecuta el workflow de release
- THEN el workflow falla con un mensaje que indica la incoherencia
- AND no se publica ningún artefacto, imagen ni release

#### Scenario: Versión de backend/Cargo.toml olvidada en el bump

- GIVEN `[workspace.package] version` y `frontend/package.json` en `0.6.0` pero `backend/Cargo.toml` todavía en `0.5.0`
- WHEN se ejecuta el workflow de release para el tag `v0.6.0`
- THEN la comprobación falla señalando `backend/Cargo.toml`
- AND no se publica ningún artefacto, imagen ni release

### Requirement: Permisos y provenance del workflow

El workflow SHALL declarar permisos mínimos: `contents: write` (crear la GitHub
Release y subir artefactos) y `packages: write` (publicar en GHCR), usando el
`GITHUB_TOKEN` integrado. Adicionalmente SHALL adjuntar *provenance* de build de
los artefactos con `actions/attest-build-provenance`; la firma criptográfica de
la imagen queda fuera de alcance en este change.

#### Scenario: Permisos suficientes y acotados

- GIVEN una ejecución del workflow de release
- WHEN se inspeccionan sus permisos
- THEN declara `contents: write` y `packages: write`
- AND no usa credenciales distintas del `GITHUB_TOKEN` para publicar

#### Scenario: Provenance adjunta a los artefactos

- GIVEN los artefactos binarios generados en el workflow
- WHEN se completa el job de release
- THEN los artefactos cuentan con una atestación de provenance de build
- AND la release sigue siendo consumible con o sin verificación de la atestación

### Requirement: Descarga acotada de artefactos en el job de release

El job `release` SHALL descargar **solo** los artefactos binarios del run
(`pattern: oximeal-*`), para que un artefacto de caché de Docker Buildx
(`*.dockerbuild`) no rompa la descarga.

#### Scenario: Descarga acotada a los binarios

- GIVEN un run de release que publica artefactos binarios `oximeal-<target>` y un artefacto de caché de Docker Buildx (`*.dockerbuild`)
- WHEN el job de release descarga los artefactos con `pattern: oximeal-*`
- THEN descarga solo los `.tar.gz` y sus `.sha256`
- AND la presencia del artefacto de caché de Docker no rompe la descarga
