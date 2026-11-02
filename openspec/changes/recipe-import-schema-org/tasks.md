# Tasks — Importación de recetas desde URL (Schema.org) (TDD: RED → GREEN → REFACTOR)

## 1. Preparación
- [ ] 1.1 Revisar `backend/src/models/recipe.rs` (`RecipeInput`, `source_url`), las rutas de `backend/src/routes/recipes.rs` y el modelo de error existente
- [ ] 1.2 Revisar `reqwest` (0.13) y `wiremock` en `backend/Cargo.toml`; confirmar que no hacen falta dependencias nuevas
- [ ] 1.3 Recopilar fixtures reales de JSON-LD `Recipe` (raíz, `@graph`, `HowToStep`/`HowToSection`, tiempos ISO-8601 y `recipeYield` variado)

## 2. RED
- [ ] 2.1 Test unitario del parser: JSON-LD `Recipe` en raíz mapea todos los campos a `RecipeInput`
- [ ] 2.2 Test unitario del parser: `Recipe` dentro de `@graph`
- [ ] 2.3 Test unitario del parser: `recipeYield` ausente → `servings` por defecto 2
- [ ] 2.4 Test unitario del parser: `PT#H#M` de `prepTime`/`totalTime` → minutos (incluye ausencia de campo)
- [ ] 2.5 Test unitario del parser: `recipeInstructions` como `HowToStep`, `HowToSection` y texto plano
- [ ] 2.6 Test SSRF: esquemas `file://`/`ftp://` → `400` (sin wiremock)
- [ ] 2.7 Test SSRF con `wiremock`: URLs a `127.0.0.1`, `localhost`, `10.x`, `169.254.x` e IPv6 `::1` → `400`
- [ ] 2.8 Test SSRF: redirección a destino prohibido → `400`
- [ ] 2.9 Test SSRF: timeout y tope de tamaño de descarga
- [ ] 2.10 Test integración `POST /api/recipes/import`: éxito → `201` con `source_url` (wiremock sirviendo HTML con JSON-LD)
- [ ] 2.11 Test integración: HTML sin receta → `422`
- [ ] 2.12 Test integración: fallo de origen (DNS/timeout/no-2xx) → `502`
- [ ] 2.13 Test integración: `Content-Type` no HTML → `400` y URL malformada → `400`
- [ ] 2.14 Test integración: idempotencia por `source_url` (segunda importación → `200`, sin duplicado)
- [ ] 2.15 Ejecutar `cargo test` y confirmar el estado RED con los tests legacy en VERDE

## 3. GREEN
- [ ] 3.1 `backend/src/domain/schema_org.rs`: localizar bloques JSON-LD, seleccionar `@type: Recipe` (incl. `@graph`) y mapear a `RecipeInput`
- [ ] 3.2 `backend/src/services/recipe_import.rs`: descarga con `reqwest` (timeout, tope de tamaño, redirecciones acotadas) y guardas SSRF (esquema, resolución y rechazo de IP prohibidas IPv4/IPv6)
- [ ] 3.3 Ruta `POST /api/recipes/import` (permiso `write:recipes`) con idempotencia por `source_url` y transacción `BEGIN IMMEDIATE`
- [ ] 3.4 Variantes de error `400`/`422`/`502` en el modelo de error
- [ ] 3.5 `cargo check` y `cargo test` en verde

## 4. REFACTOR
- [ ] 4.1 `cargo fmt --check` y `cargo clippy -- -D warnings` sin avisos
- [ ] 4.2 UI: diálogo de importación (pegar URL) en la página de recetas
- [ ] 4.3 `npx tsc --noEmit` y `pnpm test` de la UI en verde

## 5. Cierre
- [ ] 5.1 `just ci`, `just ci-ui` y `openspec validate --all --strict` en verde
- [ ] 5.2 Actualizar `PLAN.md` §10 y `CHANGELOG.md`; documentar el endpoint en el OpenAPI
- [ ] 5.3 Archivar el change con `openspec archive recipe-import-schema-org`
