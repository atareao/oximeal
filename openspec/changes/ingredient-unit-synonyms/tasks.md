# Tasks

## 1. Backend: sinónimos de unidades (TDD)
- [ ] 1.1 RED: tests con «150 gramos de pechuga de pollo» (150/`g`/«pechuga de pollo»), «2 litros de leche» (2/`l`/«leche»), «3 unidades de huevo» (3/`ud`/«huevo»), «60 gramos de brócoli», «50 gramos de copos de maíz» y sinónimos `kilo/kilogramos`, `mililitros`, `cda`, `cdta`.
- [ ] 1.2 GREEN: ampliar `KNOWN_UNITS` en `ingredient_parse.rs` (sinónimos → unidad normalizada, sin conversión).
- [ ] 1.3 REFACTOR: `cargo fmt`, `cargo clippy -p oximeal-backend --all-targets -- -D warnings`, `cargo test -p oximeal-backend`.

## 2. Verificación y despliegue
- [ ] 2.1 `npx @fission-ai/openspec@1.16.0 validate --all --strict`.
- [ ] 2.2 Auditoría `@rust-reviewer`.
- [ ] 2.3 Reconstruir la imagen `ghcr.io/atareao/oximeal:local` y recrear el contenedor `oximeal`; verificar reimportando la receta.
