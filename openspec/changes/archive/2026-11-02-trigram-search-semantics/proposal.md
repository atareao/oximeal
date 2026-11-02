# Change: Documentar la semántica de coincidencia de la búsqueda (tokenizer trigram)

## Why
El buscador envuelve `q` entre comillas y lo envía a `recipes_fts MATCH`. Con el tokenizer `trigram`, una consulta entre comillas (frase) coincide con una **subcadena contigua** del texto indexado, no con un AND de palabras: "lentejas zanahoria" NO encuentra "lentejas estofadas con zanahoria", mientras que "estofadas con" sí. Con menos de 3 caracteres se cae a `LIKE '%q%'` sobre el nombre. Esta expectativa de precisión no está documentada (issue #6).

## What Changes
- Documentar la semántica real (medida empíricamente) en el docstring de `repo::recipes::list` y en una nota nueva `docs/search.md`.
- Añadir tests de caracterización que fijen el comportamiento actual: subcadena contigua multi-palabra, palabras no contiguas no coinciden, y `<3` caracteres → `LIKE` sobre el nombre.

## Impact
- Capability: `recipe-search` (requisito documental añadido; sin cambio de comportamiento).
- Ficheros: `backend/src/repo/recipes.rs` (docstring), `backend/tests/recipes.rs` (tests), `docs/search.md` (nuevo).
