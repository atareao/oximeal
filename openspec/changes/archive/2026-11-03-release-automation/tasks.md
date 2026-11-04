# Tasks — Automatización de releases (binarios, imagen y GitHub Release)

> Metodología: SDD + TDD. Este change es de **CI/infra**, sin código de producto:
> la «verificación» es la ejecución real del workflow sobre un tag de prueba
> (o `workflow_dispatch`) y la comprobación de artefactos, imagen e informe de la
> GitHub Release. No se escribe código de runtime.

## 1. Preparación
- [x] 1.1 Confirmar la versión de trabajo: `[workspace.package] version` en `Cargo.toml`, `version` en `frontend/package.json` y última versión de `CHANGELOG.md` coinciden
- [x] 1.2 Leer `.github/workflows/ci.yml` y decidir el mecanismo de reutilización de puertas (`workflow_call`) sin alterar su disparo en `push`/`pull_request`
- [x] 1.3 Revisar el change E1 `container-image` (Dockerfile, nombre de imagen y `build-args`) para reutilizar el artefacto/receta de imagen
- [x] 1.4 Fijar ADR-33: distribución (binario portable + imagen GHCR), versionado (fuente de verdad y relación tag/versiones/CHANGELOG) y generación de notas con `git-cliff`
- [x] 1.5 Decidir la arquitectura secundaria: runner nativo ARM vs `cross`, y justificarlo en el ADR-33

## 2. RED (definir el contrato y verificar su ausencia)
- [x] 2.1 Redactar `cliff.toml` (agrupación por tipo Conventional Commits + gitmoji, encabezado de versión) y verificar en local que `git-cliff` genera notas del rango
- [x] 2.2 Escribir la comprobación de coherencia de versionado (tag ↔ `Cargo.toml` ↔ `frontend/package.json` ↔ `CHANGELOG.md`) y confirmar que **falla** con un tag incoherente
- [x] 2.3 Confirmar que hoy **no existe** `.github/workflows/release.yml` y que un tag `v*` no publica artefactos, imagen ni release

## 3. GREEN (implementar el workflow mínimo que satisface la spec)
- [x] 3.1 `.github/workflows/release.yml`: disparo `push: tags: ['v*']` + `workflow_dispatch` (input `tag`)
- [x] 3.2 Job de puertas: reutilizar `ci.yml` (backend con/sin `embed-ui`, clippy, fmt, y frontend) como dependencia previa
- [x] 3.3 Job de artefactos: matriz `x86_64-unknown-linux-gnu` y `aarch64-unknown-linux-gnu`, build `--release --features embed-ui`, `tar.gz` + `.sha256`
- [x] 3.4 Job de imagen: publicar `ghcr.io/<owner>/oximeal:<version>` y `:latest` en GHCR con `GITHUB_TOKEN`
- [x] 3.5 Job de release: crear la GitHub Release del tag adjuntando binarios y notas generadas con `git-cliff`; atestación con `actions/attest-build-provenance`
- [x] 3.6 Verificación de coherencia de versionado como paso previo a publicar (falla con tag inválido/incoherente)
- [x] 3.7 Permisos del workflow: `contents: write` y `packages: write`
- [ ] 3.8 Probar con un tag de pre-release en un repositorio/entorno de prueba: artefactos descargables, imagen en GHCR e informe de release con notas
  > Aplazado: el e2e real (publicación) es la release **v0.5.0** al cerrar P3; aquí se verifica de forma estática (`actionlint`) y con las puertas locales.

## 4. REFACTOR (endurecer y documentar)
- [x] 4.1 Revisar nombres de artefactos, etiquetas y `concurrency` del workflow; evitar publicaciones parciales (los jobs de publicación dependen de las puertas)
- [x] 4.2 Documentar en `README.md` la sección «Releases / Publicación»: flujo merge a `main` → tag → pipeline, artefactos e imagen publicados y cómo consumirlos
- [x] 4.3 Confirmar que `release.yml` no se dispara en `pull_request` y que los PR siguen solo con `ci.yml`
- [x] 4.4 Registrar ADR-33 en `PLAN.md` (Fase 7) y anotar la decisión de notas/imagen

## 5. Cierre
- [x] 5.1 `openspec validate release-automation --strict` en verde
- [ ] 5.2 Ejecución de validación end-to-end en un tag de prueba: puertas → artefactos (x86_64 y aarch64) → imagen GHCR (`<version>` y `latest`) → GitHub Release con notas
  > Aplazado por el mismo motivo que 3.8 (evita publicar artefactos/imagen/release espurios).
- [x] 5.3 Actualizar `CHANGELOG.md` (`[No publicado]` → nueva versión) y `PLAN.md`
- [ ] 5.4 Archivar el change con `openspec archive release-automation` y cerrar el issue #53
