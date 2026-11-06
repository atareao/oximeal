# Alta de recetas desde la Agent API (Valet)

## Why

Valet (asistente externo) consume OxiMeal con un token M2M y los permisos `agent:read`, `agent:plan` y `agent:shopping`. Puede consultar recetas, fijar el plan y gestionar la compra, pero **no puede crear recetas**: la Agent API no expone alta y `POST /api/recipes` exige `write:recipes`, que el cliente M2M no tiene (y que además abriría PUT/DELETE de recetas). Sin alta, Valet no puede convertir una conversación en una receta nueva.

## What Changes

- Nuevo `POST /api/agent/recipes` que acepta el mismo `RecipeInput` que `POST /api/recipes`, aplica la misma validación y devuelve `201` + `Recipe`.
- Nuevo permiso M2M `agent:write`, exigido solo por ese endpoint (mínimo privilegio: no concede PUT/DELETE ni acceso a la API de usuario).
- Router `write_router()` en `backend/src/routes/agent.rs`, protegido en `routes/mod.rs` con `require_permission(..., "agent:write")`.
- Documentación: tabla de permisos del README, scopes del cliente M2M en `deploy/oidc-dev.md` y ejemplo de `client_credentials`.
- Tests: alta válida, validaciones (categoría, ingredientes), autorización M2M (con y sin `agent:write`), token de usuario sin el permiso y sin credenciales.

## Impact

- Specs: `agent-api` (ADDED «Alta de receta»), `agent-api-auth` (MODIFIED «Acceso máquina-a-máquina»).
- Código: `backend/src/routes/agent.rs`, `backend/src/routes/mod.rs`, `backend/src/routes/openapi.rs`, `backend/tests/agent.rs`, `backend/tests/agent_auth.rs`.
- Sin migración de BD ni cambios en el modelo de receta.
- Valet (repo aparte): requiere `services/oximeal.rs` + `tools/oximeal.rs` y el scope `agent.write` en Pocket ID; fuera del alcance de este change.
