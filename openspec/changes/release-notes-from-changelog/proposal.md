# Change: notas de release desde el CHANGELOG curado

## Why

El job `release` de `.github/workflows/release.yml` generaba el cuerpo de la
GitHub Release con `git-cliff --current` sobre el rango
`v<anterior>..v<actual>` de `main`. Pero `main` integra cada release por
*squash-merge*, así que ese rango contiene un **único commit**
(`🔖 chore(release): …`) y `git-cliff` no puede reconstruir las notas: la
release `v0.5.0` se publicó con un cuerpo de tres líneas pese a que el
`CHANGELOG.md` documenta toda la Fase 7. La verificación real de la release
(assets, imagen y notas) reveló además dos fallos previos: el job descargaba
**todos** los artefactos del run —incluido el caché de Docker Buildx— y la
acción `orhun/git-cliff-action@v4` **no admite** la entrada `output`, por lo que
no escribía `RELEASE_NOTES.md`.

El `CHANGELOG.md` ya es la **fuente de verdad** de las notas (lo verifica
`check-version.sh`, ADR-33) y está curado a mano en cada PR de release. Este
change hace que la release consuma esa sección en lugar de reconstruirla de unos
commits que el squash-merge ha colapsado. Cierra el último cabo suelto de la
Fase 7 (release `v0.5.0`).

## What Changes

- Nuevo script `.github/scripts/release-notes.sh <tag> [salida]` que extrae la
  sección `## [<version>]` de `CHANGELOG.md`, descarta los encabezados de
  subsección sin contenido y escribe el fichero de notas.
- El job `release` usa ese script en lugar de `orhun/git-cliff-action@v4`.
  `git-cliff` queda como **fallback** si el CHANGELOG no tuviera la sección (y,
  si tampoco pudiera, se escribe una nota mínima con el tag: el job no falla).
- El `upload-artifact` de los binarios pasa a un nombre explícito
  (`oximeal-<target>`) y el `download-artifact` del job `release` filtra con
  `pattern: oximeal-*`, excluyendo el artefacto de caché de Docker Buildx que
  rompía la descarga.

## Non-goals

- Cambiar el formato o la plantilla `cliff.toml` (sigue disponible para el
  fallback y para uso local).
- Firmar criptográficamente la imagen (fuera de alcance, ADR-33).
- Editar retrospectivamente el `CHANGELOG.md` de releases ya publicadas.

## Impact

- Spec afectada: `release` (requisito «GitHub Release con notas generadas»:
  MODIFIED).
- Ficheros: `.github/workflows/release.yml`, `.github/scripts/release-notes.sh`.
- Sin cambios de contrato de API ni de dependencias del frontend.
