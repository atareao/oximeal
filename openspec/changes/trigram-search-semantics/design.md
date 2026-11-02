# Design — Semántica de búsqueda del tokenizer trigram

## Context
`repo::recipes::list` envuelve la consulta en comillas como frase y la pasa a `recipes_fts MATCH` (nombre o instrucciones) si `q` tiene ≥ 3 caracteres; si no, usa `LIKE '%q%'` sobre `r.name`. El índice usa `tokenize='trigram remove_diacritics 1'`.

## Comportamiento medido (SQLite 3.54, trigram)
- **Frase = subcadena contigua:** el tokenizer produce trigramas solapados; una frase exige trigramas consecutivos, lo que equivale a buscar `q` como subcadena contigua (insensible a mayúsculas y diacríticos). Verificado: `"estofadas con"` coincide; `"con lentejas"` (orden invertido) no; `"lentejas zanahoria"` no coincide aunque ambas palabras estén presentes.
- **Un solo término corto también es subcadena:** p. ej. `"zana"` coincide dentro de "zanahoria".
- **Plegado de diacríticos:** `"cafe"` y `"Café"` coinciden ambos.
- **< 3 caracteres:** un trigrama exige 3 caracteres, así que `MATCH` no encuentra nada; de ahí el fallback a `LIKE`.

## Decision
Documentar como *expectativa de precisión* que el buscador es una búsqueda de **subcadena contigua** (no una búsqueda por palabras/AND), para que la UI y los usuarios no esperen coincidencias no contiguas. Se fija con tests de caracterización (no cambia el código de producción salvo el docstring).

## Testing
- **Integración** (`backend/tests/recipes.rs`): subcadena contigua multi-palabra devuelve la receta; palabras no contiguas → `total = 0`; un término de 3 caracteres presente solo en instrucciones se encuentra (FTS); y un término de `<3` caracteres **no** busca en instrucciones (el `LIKE` cae solo sobre el nombre).
