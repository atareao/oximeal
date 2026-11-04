# Change: actualización automatizada de dependencias (Dependabot)

## Why

El repositorio no tiene ningún mecanismo automatizado de actualización de
dependencias: `Cargo.toml`/`Cargo.lock`, `frontend/package.json` con
`pnpm-lock.yaml`, las acciones de los workflows y la imagen base del `Dockerfile`
se actualizan **a mano**. Al cerrar la Fase 7 (hotfix #69) hubo que subir diez
acciones a mayores con Node 24 precisamente porque nadie avisa de las
deprecaciones ni de las versiones nuevas.

Dependabot solo lee su configuración de la **rama por defecto** (`main`), pero en
este repositorio el flujo es gitflow: `main` solo recibe releases y hotfixes, y
todo lo demás entra por `development`. Por eso la configuración debe vivir en
`main` **apuntando sus PRs a `development`**.

## What Changes

- Nuevo `.github/dependabot.yml` (version 2) con cuatro ecosistemas:
  - `cargo` en `/` (workspace de la raíz, miembro `backend`);
  - `npm` en `/frontend` (pnpm, `pnpm-lock.yaml`);
  - `github-actions` en `/` (`.github/workflows/`);
  - `docker` en `/` (`Dockerfile`).
- Todos con `target-branch: development`, `schedule: weekly` (lunes), un límite
  de PRs abiertas, la etiqueta `dependencies` y `commit-message.prefix` con
  gitmoji (`⬆️ deps`).
- Grupos `minor`+`patch` por ecosistema; las actualizaciones `major` quedan en
  PRs separadas para revisarlas una a una.

## Non-goals

- No se activa *auto-merge*: cada PR sigue pasando por las puertas de `ci.yml` y
  por revisión humana.
- No se cambia ninguna dependencia en este change (solo se habilita la
  automatización).

## Impact

- Capability nueva `dependencies` (solo afecta a mantenimiento; nada de producto).
- Ficheros: `.github/dependabot.yml`.
