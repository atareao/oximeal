# Change: el generador de notas de release se toma del ref de ejecución

## Why

Al validar por primera vez el pipeline corregido (opción **A**,
`workflow_dispatch tag=v0.5.0`, run `41789091012`) el job `release` falló en el
step nuevo con `exit 127`:

```
bash: .github/scripts/release-notes.sh: No such file or directory
```

Causa raíz: el job `release` hace `actions/checkout` con
`ref: ${{ inputs.tag || github.ref }}`, es decir, **el árbol del tag**. Cuando se
revalida un tag ya publicado (o, en general, cuando el tag es anterior a un
arreglo del generador o del `CHANGELOG.md`), ese árbol no contiene
`.github/scripts/release-notes.sh` y el job muere *después* de publicar binarios
e imagen. El tag `v0.5.0` (`f332e04`) solo tiene `check-version.sh` en
`.github/scripts/`, lo que confirma el diagnóstico.

El generador de notas y la sección curada del `CHANGELOG.md` pertenecen a la
**definición del workflow** (el ref que dispara la ejecución), no al tag que se
está publicando.

## What Changes

- En el job `release` (y **solo** en él), el checkout pasa a usar el commit que
  disparó la ejecución (`github.sha`, vía `ref: ${{ github.sha }}`) en vez del
  tag, de modo que `.github/scripts/release-notes.sh` y `CHANGELOG.md` salen del
  ref del run.
- Los jobs `artifacts` e `image` **no cambian**: siguen construyendo desde el tag
  (`inputs.tag || github.ref`), que es lo correcto (se publica el código
  etiquetado).
- Se alinea el `Purpose` de la capability con el comportamiento ya vigente: las
  notas se toman de la sección curada del `CHANGELOG.md` (git-cliff solo como
  *fallback* no fatal).

## Non-goals

- No se cambia de qué ref compilan los jobs `artifacts`/`image`.
- No se cambia ningún aspecto del producto (ni API, ni UI, ni esquema).

## Impact

- Capability `release`: MODIFIED «GitHub Release con notas generadas».
- Ficheros: `.github/workflows/release.yml`, `openspec/specs/release/spec.md`
  (`Purpose`).
