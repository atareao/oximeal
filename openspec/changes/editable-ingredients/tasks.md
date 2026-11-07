# Tasks

## 1. Backend: parseo de ingredientes importados (TDD)
- [ ] 1.1 RED: tests del helper de parseo con «40 g de queso parmesano», «1 calabacín», «1/2 taza de harina», «sal al gusto», decimal con coma y mixta «1 1/2».
- [ ] 1.2 GREEN: implementar `parse_ingredient` y conectarlo en `schema_org::map_recipe`.
- [ ] 1.3 REFACTOR: actualizar el test existente que fijaba nombre completo + `1 ud`; `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.

## 2. Frontend: edición de ingredientes en el detalle (TDD)
- [ ] 2.1 RED: tests de `RecipeDetailPage` que editan cantidad/unidad de un ingrediente, añaden/quitan y validan.
- [ ] 2.2 GREEN: edición inline reutilizando `useUpdateRecipe` (`PUT /api/recipes/{id}`).
- [ ] 2.3 REFACTOR: `tsc --noEmit`, `eslint`, `vitest run`, y ajustar estilos/accesibilidad (objetivos ≥48 px).

## 3. Verificación y documentación
- [ ] 3.1 `npx @fission-ai/openspec@1.16.0 validate --all --strict`.
- [ ] 3.2 Auditoría: `@rust-reviewer` (backend) y `@react-reviewer` (frontend).
- [ ] 3.3 `docs/editable-ingredients.md` y archivar plan/change.
