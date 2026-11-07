# PLAN-001 — Comensales habituales y escalado en el detalle

- **Change SDD:** `household-servings`
- **Estado:** entregado (PR #104 mergeado)
- **Fecha:** 2026-11-07

## Objetivo

Permitir configurar los comensales habituales del hogar y escalar las cantidades
del detalle de receta a las raciones seleccionadas, sin tocar el cálculo de la
lista de la compra.

## Tareas

- [x] Spec: `household-settings` (nueva) + `recipes` (MODIFIED), validada strict.
- [x] Migración `0010_household_settings.sql` (singleton, CHECK 1..20, seed).
- [x] Backend: modelo, repo, rutas `GET/PUT /api/settings`, permisos, OpenAPI.
- [x] Tests backend: contrato, fronteras, tipos, persistencia, CHECKs, OpenAPI.
- [x] Frontend: `lib/scaling.ts`, API/hooks, persistencia offline de `settings`.
- [x] Ajustes: campo «Comensales habituales» con guardado y errores.
- [x] Detalle: selector único, escalado, aviso, «Ver originales», importadas.
- [x] Tests frontend: escalado, ajustes, detalle, navegación A→B, clamp.
- [x] Auditorías `@rust-reviewer` (PASS) y `@react-reviewer` (FAIL → corregido).
- [x] Documentación `docs/household-servings.md`.
- [x] PR a `development` y merge (#104).

## Puertas

`cargo test --tests` (salvo los 2 de `config` por `.env` local), `cargo clippy -D warnings`,
`cargo fmt --check`, `npx vitest run` (373), `npx tsc --noEmit`, `npm run lint`,
`openspec validate --all --strict`.
