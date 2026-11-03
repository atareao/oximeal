# Semántica de la búsqueda de recetas

La búsqueda del listado (`GET /api/recipes?q=...`) coincide con `q` como
**subcadena contigua** (insensible a mayúsculas y a diacríticos) del **nombre**
o de las **instrucciones** de una receta, cuando `q` tiene **3 o más
caracteres**. **No** es una búsqueda por palabras ni un AND de términos.

## Por qué

El índice `recipes_fts` usa `tokenize='trigram remove_diacritics 1'` y el
término se envía entre comillas (frase). El tokenizer `trigram` genera trigramas
solapados; una frase exige trigramas consecutivos, lo que equivale a una
subcadena contigua del texto.

## Ejemplos (medidos)

| Consulta `q`            | ¿Coincide con "lentejas estofadas con zanahoria"? |
|---|---|
| `estofadas con`         | Sí (subcadena contigua) |
| `lentejas zanahoria`    | **No** (las palabras no son contiguas) |
| `zanaho`                | Sí (subcadena dentro de una palabra) |
| `cafe` (documento "Café") | Sí (los diacríticos se pliegan) |

## Menos de 3 caracteres

El tokenizer `trigram` exige 3 caracteres por trigrama, así que `MATCH` no
puede resolver consultas de 1 o 2 caracteres. En ese caso la búsqueda cae a
`LIKE '%q%'` sobre el **nombre** de la receta (no busca en instrucciones).

A diferencia de la ruta FTS, este `LIKE` **no pliega diacríticos** y su
insensibilidad a mayúsculas es solo para ASCII. Es una vía de escape para
términos muy cortos, no una búsqueda totalmente equiparable.

## Implicación para la UI

Si se quiere ofrecer búsqueda "por palabras" (p. ej. `lentejas zanahoria`),
habría que dividir el término en tokens y combinarlos (por ejemplo uniendo por
`AND` varias frases FTS), o cambiar la estrategia de indexado. Mientras tanto,
la UI debe presentar el buscador como búsqueda de texto contiguo.

## Filtro por etiqueta

Las etiquetas quedan **fuera** de `recipes_fts`: buscar su término con `q` no
encuentra las recetas etiquetadas con él. Para localizarlas hay que usar el
filtro dedicado `GET /api/recipes?tag=<id|slug>`, que se combina con `q`,
`category` y la paginación, y cuyo `total` refleja el conjunto filtrado.
