# Capability: recipe-search

## ADDED Requirements

### Requirement: Semántica de coincidencia de la búsqueda
El sistema SHALL coincidir con `q` como **subcadena contigua** (insensible a mayúsculas y a diacríticos) de `recipes.name` o `recipes.instructions` cuando `q` tenga 3 o más caracteres, y SHALL caer a `LIKE '%q%'` sobre `recipes.name` cuando tenga menos de 3. La expectativa de precisión SHALL quedar documentada (`docs/search.md` y docstring de `repo::recipes::list`).

#### Scenario: Subcadena contigua multi-palabra
- GIVEN una receta cuyas instrucciones contienen "estofadas con zanahoria"
- WHEN se consulta `GET /api/recipes?q=estofadas con`
- THEN se devuelve la receta

#### Scenario: Palabras no contiguas no coinciden
- GIVEN una receta cuyas instrucciones contienen "lentejas estofadas con zanahoria"
- WHEN se consulta `GET /api/recipes?q=lentejas zanahoria`
- THEN el resultado es vacío (no es subcadena contigua)

#### Scenario: Menos de 3 caracteres usa LIKE sobre el nombre
- GIVEN una receta llamada "Pollo asado"
- WHEN se consulta `GET /api/recipes?q=ll`
- THEN se devuelve la receta
