# Tareas — agent-create-recipe

## 1. RED

- [x] 1.1 `backend/tests/agent.rs`: alta válida devuelve 201 y la receta aparece en `GET /api/recipes`.
- [x] 1.2 `backend/tests/agent.rs`: categoría inválida y cero ingredientes devuelven 400.
- [x] 1.3 `backend/tests/agent_auth.rs`: token M2M con `agent:write` crea (201); sin `agent:write` devuelve 403; token de usuario devuelve 403; sin credenciales 401.
- [x] 1.4 Ejecutar `cargo test`; confirmar rojo por la causa esperada.

## 2. GREEN

- [x] 2.1 `write_router()` con `POST /api/agent/recipes` en `backend/src/routes/agent.rs`.
- [x] 2.2 Handler que reutiliza `RecipeInput` y la validación de `POST /api/recipes`; devuelve 201.
- [x] 2.3 Proteger el router con `require_permission(..., "agent:write")` en `backend/src/routes/mod.rs`.
- [x] 2.4 Registrar el path en OpenAPI.
- [x] 2.5 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt` y `clippy -D warnings`.
- [x] 3.2 Revisar que la validación no esté duplicada (extraer helper si hace falta).
- [x] 3.3 `just ci`.

## 4. Documentación y cierre

- [x] 4.1 README: tabla de permisos y ejemplo M2M con `agent:write`.
- [x] 4.2 `deploy/oidc-dev.md`: scope `agent.write` del cliente Valet.
- [x] 4.3 `openspec validate --all --strict`.
- [x] 4.4 Archivar el change.
- [x] 4.5 CHANGELOG y PR a `development`.

Al terminar, ejecutar `npx --yes @fission-ai/openspec@1.16.0 validate agent-create-recipe --strict` y devuelve solo OK o los errores.