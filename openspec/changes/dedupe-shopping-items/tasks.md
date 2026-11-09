# Tasks

## 1. Backend — RED (tests)

- [ ] 1.1 Test de reconciliación: dos líneas manuales «Tomate» 2 ud y 3 ud → una de 5 ud.
- [ ] 1.2 Test de unidades compatibles: «Harina» «1 kg» y «500 g» → una línea equivalente a 1500 g.
- [ ] 1.3 Test de unidades incompatibles: «Tomate» «2 ud» y «500 g» → dos líneas.
- [ ] 1.4 Test de orígenes: manual vs plan con la misma clave → dos líneas.
- [ ] 1.5 Test de `checked` y determinismo del supervivente (`id` menor).
- [ ] 1.6 Test de idempotencia y marca persistente (segunda ejecución no cambia nada).
- [ ] 1.7 Test de orden determinista del listado (`GET /api/shopping-list`).
- [ ] 1.8 Verificar RED.

## 2. Backend — GREEN

- [ ] 2.1 Migración `0012_shopping_backfills.sql` (tabla `shopping_backfills`).
- [ ] 2.2 `services::shopping_reconcile::reconcile(pool)` con marcador.
- [ ] 2.3 Llamada en `main.rs` tras el backfill de autotag (tolerante a fallos).
- [ ] 2.4 Verificar GREEN.

## 3. Backend — REFACTOR

- [ ] 3.1 `cargo fmt --check` y `cargo clippy -- -D warnings`.
- [ ] 3.2 `openspec validate dedupe-shopping-items --strict`.

## 4. Auditoría

- [ ] 4.1 Revisión con `@rust-reviewer`.

## 5. Documentación y cierre

- [ ] 5.1 `docs/shopping-list-dedupe.md`.
- [ ] 5.2 Archivar el change y consolidar la spec.
- [ ] 5.3 PR a `development`, CI en verde y merge.
