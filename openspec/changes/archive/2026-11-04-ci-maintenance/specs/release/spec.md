# Capability: release

## MODIFIED Requirements

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
