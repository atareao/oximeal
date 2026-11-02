# Change: importar recetas desde una URL (Schema.org JSON-LD)

## Why

`PLAN.md` §10 declara la deuda: «Fase 1 acepta solo JSON validado; el parser
Schema.org se completa en F2/F3». Hoy el recetario solo se llena a mano o por el
agente, así que incorporar una receta vista en la web obliga a copiar cada campo.
Pegar una URL debería bastar.

Muchas páginas de recetas publican metadatos estructurados
(`<script type="application/ld+json">` con `@type: Recipe`) que contienen nombre,
ingredientes, raciones, tiempos e instrucciones. El navegador **no puede** leer
ese HTML de terceros por CORS ni por la política de contenido, de modo que la
descarga y el parseo SHALL ocurrir en el backend, que además puede aplicar
guardas de red (SSRF).

Trazabilidad: issue #46.

## What Changes

- Nuevo endpoint `POST /api/recipes/import` (permiso `write:recipes`) que acepta
  `{ "url": "<http(s)>" }`.
- Guardas SSRF antes de conectar: solo esquemas `http`/`https`, resolución del
  host y rechazo de rangos privados, loopback, link-local y `0.0.0.0` (IPv4 e
  IPv6), límite de redirecciones sin permitir saltos a direcciones prohibidas,
  timeout de conexión/lectura y tope de tamaño de descarga.
- Parseo de los bloques JSON-LD embebidos: localiza el objeto `@type: Recipe`
  (también dentro de `@graph`) y lo mapea a `RecipeInput`:
  `name` → nombre, `recipeIngredient` → ingredientes, `recipeYield` → `servings`
  (por defecto 2), `prepTime`/`totalTime` (ISO-8601 `PT#H#M`) → minutos,
  `recipeInstructions` (`HowToStep`/`HowToSection`/texto) → instrucciones,
  `url` → `source_url`.
- Persistencia idempotente por `source_url`: si ya existe una receta con la misma
  `source_url` no se duplica.
- UI: diálogo de importación (pegar URL) en la página de recetas.

## Impact

- Capabilities: `recipes` (delta ADDED con tres requisitos nuevos).
- Backend: nuevos módulos (p. ej. `backend/src/domain/schema_org.rs` para el
  parser y `backend/src/services/recipe_import.rs` para descarga+guardas),
  rutas en `backend/src/routes/recipes.rs`, variantes nuevas en el modelo de
  error.
- Frontend: diálogo de importación en la página de recetas.
- Sin dependencias nuevas: `reqwest` (0.13, json/rustls/form) ya está en el
  backend y `wiremock` en dev-dependencies para los tests.
