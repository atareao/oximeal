# release Specification

## Purpose

Esta capability define la **automatización de releases** de OxiMeal: un workflow
`.github/workflows/release.yml` disparado por tags `v<version>` (o por
`workflow_dispatch`) que, tras reutilizar las puertas de calidad de `ci.yml` vía
`workflow_call`, publica artefactos binarios `x86_64`/`aarch64` (`.tar.gz` +
`.sha256`), una imagen en GHCR (`<version>` y `latest`, sin retroceder con
pre-releases) y una GitHub Release con notas generadas por `git-cliff`
(`cliff.toml`) a partir de Conventional Commits + gitmoji, con provenance de
build. Incluye la verificación de coherencia entre el tag, `[workspace.package]
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

El workflow SHALL ejecutar las mismas puertas de calidad que `ci.yml` —`cargo
fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings` y
`cargo test` con y sin la feature `embed-ui`, más las puertas del frontend
(`typecheck`, `format:check`, `test`, `lint`, `build`, `check-bundle`)— como
dependencia previa de los jobs de publicación. Si cualquier puerta falla, el
workflow SHALL terminar sin publicar artefactos, imagen ni release.

#### Scenario: Puertas en verde habilitan la publicación

- GIVEN un tag de versión válido
- WHEN se ejecuta el workflow de release
- THEN se ejecutan las puertas de backend y frontend
- AND, si todas pasan, continúan los jobs de artefactos, imagen y release

#### Scenario: Fallo de build impide publicar

- GIVEN un tag de versión válido y un fallo en `cargo test`, `clippy` o el build del frontend
- WHEN se ejecuta el workflow de release
- THEN el workflow falla antes de cualquier paso de publicación
- AND no se crea ni artefacto binario, ni imagen en GHCR, ni GitHub Release

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

El workflow SHALL crear la **GitHub Release** del tag adjuntando los artefactos
binarios y con **notas de versión generadas automáticamente** a partir de los
commits (Conventional Commits + gitmoji), usando **`git-cliff`** con la
configuración `cliff.toml` versionada en el repositorio. Las notas SHALL ser
coherentes con las secciones de `CHANGELOG.md`.

#### Scenario: Release creada con binarios y notas

- GIVEN un tag `v0.5.0` y los artefactos y la imagen publicados
- WHEN se ejecuta el job de release
- THEN se crea una GitHub Release para `v0.5.0`
- AND la release adjunta los `.tar.gz` y sus `.sha256`
- AND el cuerpo de la release contiene las notas generadas con `git-cliff` a partir de los commits del rango

#### Scenario: Sin commits publicables

- GIVEN un rango de commits que no contiene entradas de usuario relevantes (solo `chore`/`ci`)
- WHEN `git-cliff` genera las notas
- THEN la release se crea igualmente
- AND el cuerpo de la release no queda vacío de forma silenciosa (incluye al menos el encabezado de versión y la referencia al tag)

### Requirement: Coherencia de versionado (ADR-33)

La versión del proyecto SHALL tener como fuente de verdad
`[workspace.package] version` de `Cargo.toml`. El workflow SHALL verificar que el
tag, `[workspace.package] version`, `version` de `frontend/package.json` y la
sección más reciente con versión de `CHANGELOG.md` son coherentes; si no lo son,
el workflow SHALL fallar antes de publicar. El *bump* de versión SHALL ser manual
en un PR de release a `main`, y el tag SHALL tener el formato `v<version>`
(SemVer).

#### Scenario: Tag y versión coherentes

- GIVEN `Cargo.toml` con `version = "0.5.0"`, `frontend/package.json` con `"version": "0.5.0"` y el tag `v0.5.0`
- WHEN se ejecuta el workflow de release
- THEN la comprobación de coherencia pasa
- AND el workflow continúa con la publicación

#### Scenario: Tag inválido o incoherente

- GIVEN un tag que no empieza por `v`, con formato no SemVer, o cuyo número no coincide con la `version` del workspace
- WHEN se ejecuta el workflow de release
- THEN el workflow falla con un mensaje que indica la incoherencia
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
