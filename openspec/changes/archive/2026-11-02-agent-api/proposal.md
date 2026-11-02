# Change: Agent API (contrato para Valet)

## Intent
Exponer una superficie HTTP estable y tipada en `/api/agent` para que el asistente Valet ejecute acciones sobre OxiMeal (consultar recetas recientes, buscar recetas, fijar el plan semanal y gestionar la lista de la compra) sin que OxiMeal necesite orquestador LLM, MCP ni chat propios.

## Scope
- `GET /api/agent/recent-recipes?days=`
- `GET /api/agent/search-recipes?query=&category=`
- `POST /api/agent/weekly-plan` (bulk transaccional + reconstrucción de la compra)
- `POST /api/agent/shopping-items` (inserta o suma cantidades)
- `DELETE /api/agent/shopping/completed`
- Documentación en OpenAPI y tests de integración.

## Out of scope
- Orquestador LLM, servidor MCP y chat (decisión: el asistente es Valet).
- Integración en el repo Valet (se hará allí).

## Impact
- Nueva capacidad `agent-api`; sustituye a las "Tools MCP" del §4 de la especificación original por endpoints HTTP equivalentes.
