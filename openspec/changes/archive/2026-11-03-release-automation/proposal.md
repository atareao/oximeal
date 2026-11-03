# Change: automatización de releases (binarios, imagen y GitHub Release)

## Why

OxiMeal se distribuye como **binario único** (Axum + UI embebida con la feature
`embed-ui`) e **imagen Docker**, pero hoy no existe ningún pipeline que publique
nada: `.github/workflows/ci.yml` solo valida `push`/`pull_request` a `main` y
`development`, y el *badge* `Release` del `README.md` apunta a una página de
GitHub Releases que se actualiza **a mano**. El `CHANGELOG.md` ya sigue Keep a
Changelog + SemVer y la `version` del workspace (`[workspace.package] version`) y
la de `frontend/package.json` están sincronizadas a `0.4.0`, pero no hay nada que
garantice esa coherencia, ni que compile artefactos, ni que publique la imagen o
las notas de versión.

Falta por tanto el eslabón de **entrega (Fase 7)**: etiquetar una release debe
producir, de forma reproducible y verificada, artefactos binarios descargables,
una imagen en GHCR y una GitHub Release con notas generadas a partir de los
commits (Conventional Commits + gitmoji). Sin pipeline, cada release es un
proceso manual frágil: se olvida la puerta de calidad, se publica una imagen sin
correspondencia con el tag o las notas se escriben a mano y divergen del
historial. Este change cierra el issue #53 (change **E2** de la Fase 7, entrega)
y concreta el **ADR-33** (estrategia de distribución y versionado), coordinado
con la imagen GHCR definida en el change **E1** `container-image`.

## What Changes

- **Nuevo workflow `.github/workflows/release.yml`**, disparado por **tags `v*`**
  y, opcionalmente, por `workflow_dispatch` (input `tag`). No se ejecuta en
  `pull_request`: los PR siguen cubiertos por `ci.yml`.
- **Puertas de calidad antes de publicar**: el job de release **reutiliza el
  workflow `ci.yml`** (vía `workflow_call`) o repite sus jobs (backend con y sin
  `embed-ui`, clippy, `fmt`, frontend) como dependencia previa; si alguna falla,
  **no se publica nada**.
- **Artefactos binarios**: matriz de build para `x86_64-unknown-linux-gnu` y
  `aarch64-unknown-linux-gnu` (runner nativo de ARM o `cross`; se decide y
  justifica), compilados con `--release --features embed-ui`, empaquetados en
  `.tar.gz` con su `.sha256` y subidos como artefactos del workflow.
- **Imagen en GHCR**: se publica `ghcr.io/<owner>/oximeal` con la etiqueta
  **semver** (derivada del tag) y `latest`, reutilizando la imagen del change E1.
- **GitHub Release**: se crea la release del tag adjuntando los binarios y con
  **notas generadas** desde Conventional Commits + gitmoji mediante **`git-cliff`**
  (config en el repo, coherente con `GIT_FLOW.md`; se descarta `svu`/`vampus` por
  no añadir un servicio externo). Las notas se coordinan con `CHANGELOG.md`.
- **Versionado (ADR-33)**: se documenta cómo se relacionan el **tag**,
  `[workspace.package] version`, `frontend/package.json` y `CHANGELOG.md`. La
  versión es la fuente de verdad en `Cargo.toml`; el tag DEBE ser `v<version>` y
  el workflow SHALL verificar la coherencia (si no coincide → falla). El *bump* es
  **manual** en un PR de release a `main` (rama `release/*`); tras el merge se
  crea el tag, que dispara el pipeline.
- **Permisos y trazabilidad**: `permissions: contents: write` (crear la release) y
  `packages: write` (publicar en GHCR), usando el `GITHUB_TOKEN` integrado. Se
  adjunta *provenance* de build (`actions/attest-build-provenance`) de forma
  pragmática; la firma de imágenes se deja fuera de alcance por ahora.
- **Documentación**: se añade una sección «Releases / Publicación» al `README.md`
  con el flujo (merge a `main` → tag → pipeline), los artefactos publicados y cómo
  consumirlos.

## Impact

- Nuevo: `.github/workflows/release.yml` y `cliff.toml` (configuración de notas).
- Modificado: `.github/workflows/ci.yml` (exposición `workflow_call` para reutilizar
  las puertas, sin cambiar su comportamiento en `push`/`pull_request`), `README.md`
  (sección «Releases / Publicación»), `CHANGELOG.md` (nueva entrada en `[No publicado]`
  al archivarse) y `PLAN.md` (Fase 7 / ADR-33).
- Capability: **nueva** `release`.
- Reutiliza el binario `oximeal-backend` con `--features embed-ui` y la imagen GHCR
  del change E1 `container-image`; sin dependencias de runtime nuevas (solo
  herramientas de CI: `git-cliff` y, si se opta por matriz ARM, `cross`/runner ARM).
- Cierra el issue **#53**; implementa el **ADR-33**.
