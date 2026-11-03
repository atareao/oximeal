# Capability: recipe-search

## MODIFIED Requirements

### Requirement: Índices de rendimiento

El sistema SHALL definir los índices `idx_recipes_created_at` sobre
`recipes(created_at)`, `idx_shopping_source_plan_id` sobre
`shopping_list_items(source_plan_id)`, `idx_shopping_checked` sobre
`shopping_list_items(checked)`, el índice `idx_recipe_tags_tag_id` sobre
`recipe_tags(tag_id)` y la restricción de unicidad sobre `tags(slug)`, para
acelerar las consultas frecuentes y garantizar la identidad canónica de las
etiquetas. Las etiquetas NO SHALL incorporarse al índice de texto completo
`recipes_fts`.

#### Scenario: Existencia de índices

- GIVEN una base migrada
- WHEN se consulta `sqlite_master` para el tipo índice
- THEN existen `idx_recipes_created_at`, `idx_shopping_source_plan_id` e `idx_shopping_checked`

#### Scenario: Índices de etiquetas

- GIVEN una base migrada hasta `0007`
- WHEN se consulta `sqlite_master`
- THEN existen `idx_recipe_tags_tag_id` y la restricción `UNIQUE` sobre `tags(slug)`

#### Scenario: Las etiquetas no contaminan la búsqueda de texto

- GIVEN una receta etiquetada con un término que no aparece en su nombre ni en sus instrucciones
- WHEN se busca ese término con `GET /api/recipes?q=<término>`
- THEN la receta no aparece por coincidencia de texto completo
