# recipes Specification

## Purpose
Esta capacidad cubre la gestión de recetas: creación, consulta, actualización y borrado (CRUD), así como el listado y la búsqueda de recetas. La búsqueda combina SQLite FTS5 con un fallback por LIKE y permite filtrar por categorías.

## Requirements

### Requirement: Listado y búsqueda de recetas

El sistema SHALL exponer `GET /api/recipes` con filtro de texto opcional `q`
(FTS5 trigram), filtro `category` y filtro `tag` (id o slug de etiqueta), con
paginación. El `total` de la respuesta SHALL reflejar el conjunto que cumple los
filtros aplicados.

#### Scenario: Búsqueda por substring

- GIVEN recetas que contienen "tomate" en nombre o instrucciones
- WHEN se consulta `GET /api/recipes?q=tomate`
- THEN se devuelven las recetas coincidentes con metadatos de paginación

#### Scenario: Query corta

- GIVEN una query de menos de 3 caracteres
- WHEN se consulta el listado
- THEN el sistema usa fallback LIKE en lugar de MATCH

#### Scenario: Filtro por etiqueta

- GIVEN recetas asociadas a la etiqueta `sin-gluten`
- WHEN se consulta `GET /api/recipes?tag=sin-gluten`
- THEN se devuelven solo las recetas asociadas a esa etiqueta
- AND el `total` refleja ese subconjunto

#### Scenario: Filtros combinados

- GIVEN recetas que cumplen parte de una búsqueda de texto y de un filtro de categoría y etiqueta
- WHEN se consultan `q`, `category` y `tag` a la vez
- THEN se devuelven solo las recetas que cumplen todos los criterios, paginadas

### Requirement: Alta de receta

El sistema SHALL exponer `POST /api/recipes` que valida categoría (main|salad|side|dessert|breakfast) e ingredientes, persiste la receta y SHALL aceptar un campo opcional `tags` con los nombres de sus etiquetas manuales, creando al vuelo las que no existan. SHALL aplicar el etiquetado automático salvo que `auto_tags` sea `false` y SHALL aceptar `suppressed_auto_tags`. La `Recipe` devuelta SHALL incluir sus etiquetas con `source`.

#### Scenario: Creación válida

- GIVEN un payload de receta válido
- WHEN se envía a `POST /api/recipes`
- THEN se crea la receta con id y se devuelve 201

#### Scenario: Categoría inválida

- GIVEN un payload con categoría no permitida
- WHEN se envía
- THEN se devuelve 400 y no se persiste nada

#### Scenario: Alta con etiquetas

- GIVEN un payload válido con `tags: ["Vegano"]` y la etiqueta no existía
- WHEN se envía a `POST /api/recipes`
- THEN se crea la etiqueta y la receta queda asociada a ella como `manual`
- AND la `Recipe` devuelta incluye la etiqueta y las automáticas resultantes

#### Scenario: Alta sin etiquetado automático

- GIVEN un payload válido con `auto_tags: false`
- WHEN se envía a `POST /api/recipes`
- THEN la receta no recibe etiquetas automáticas
- AND solo se asocian las etiquetas manuales indicadas

### Requirement: Detalle de receta

El sistema SHALL exponer `GET /api/recipes/{id}` con la receta completa, sus ingredientes y etiquetas (`id`, `name`, `slug`, `source`). El detalle SHALL escalar las cantidades a las raciones seleccionadas (1..20), usando la preferencia de comensales como valor inicial, con aviso del cálculo y opción de ver las originales. Si la receta no trae cantidades reales (todas `amount = 1` y `unit = "ud"`), SHALL mostrar las originales con aviso y no escalar.

#### Scenario: No encontrada

- GIVEN un id inexistente
- WHEN se consulta el detalle
- THEN se devuelve 404

#### Scenario: Detalle con etiquetas

- GIVEN una receta con etiquetas manuales y automáticas
- WHEN se consulta `GET /api/recipes/{id}`
- THEN la respuesta incluye las etiquetas con `id`, `name`, `slug` y `source`

#### Scenario: Escalado por defecto

- GIVEN una receta para 8 y una preferencia de 2 comensales
- WHEN se abre el detalle
- THEN el selector muestra 2
- AND las cantidades se multiplican por 1/4
- AND se muestra «Receta para 8 · cantidades para 2»

#### Scenario: Cambio manual de raciones

- GIVEN el detalle abierto
- WHEN el usuario selecciona 6 raciones
- THEN las cantidades se recalculan con factor 6/8
- AND el aviso refleja 6

#### Scenario: Ver cantidades originales

- GIVEN el detalle con cantidades escaladas
- WHEN el usuario activa «Ver originales»
- THEN se muestran las cantidades de la receta sin factor

#### Scenario: Receta importada sin cantidades

- GIVEN una receta importada con todos los ingredientes en `amount = 1` y `unit = "ud"`
- WHEN se abre el detalle
- THEN se muestran las cantidades originales
- AND se avisa de que la importación no trae cantidades y no se escala

### Requirement: Actualización de receta

El sistema SHALL exponer `PUT /api/recipes/{id}` con permiso `write:recipes`, que reemplaza la receta completa —nombre, categoría, `diet_type`, `prep_time_minutes`, `servings`, ingredientes, instrucciones, `source_url` y etiquetas manuales—, responde `200` con la `Recipe` actualizada y mantiene sincronizado el índice de búsqueda FTS. El campo `tags` SHALL reemplazar el conjunto de etiquetas `manual`, y el etiquetado automático SHALL recalcularse salvo que `auto_tags` sea `false`.

#### Scenario: Actualización válida

- GIVEN una receta existente y un payload válido con todos los campos
- WHEN se envía a `PUT /api/recipes/{id}`
- THEN se reemplaza la receta y se devuelve 200 con la `Recipe` actualizada
- AND el índice FTS refleja el cambio y el listado devuelve los datos nuevos

#### Scenario: Receta inexistente

- GIVEN un id que no existe
- WHEN se envía un payload válido a `PUT /api/recipes/{id}`
- THEN se devuelve 404
- AND no se crea ni modifica ninguna receta

#### Scenario: Payload inválido

- GIVEN una receta existente y un payload con categoría no permitida o campos inválidos
- WHEN se envía a `PUT /api/recipes/{id}`
- THEN se devuelve 400
- AND la receta original permanece sin cambios

#### Scenario: Actualización sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `PUT /api/recipes/{id}` sin el permiso `write:recipes`
- THEN se devuelve 401 si no hay sesión o 403 si falta el permiso
- AND no se modifica ninguna receta

#### Scenario: Reemplazo de etiquetas

- GIVEN una receta asociada manualmente a las etiquetas A y B
- WHEN se envía `PUT /api/recipes/{id}` con `tags: ["B", "C"]`
- THEN la receta queda asociada a B y C como `manual`
- AND la asociación manual con A desaparece
- AND no se borra ninguna etiqueta del catálogo

### Requirement: Borrado de receta

El sistema SHALL exponer `DELETE /api/recipes/{id}` con permiso `write:recipes`,
que borra la receta y sus ingredientes de forma atómica, responde `204` y elimina
la receta del listado `GET /api/recipes`.

#### Scenario: Borrado válido

- GIVEN una receta existente con ingredientes
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se borran la receta y sus ingredientes y se devuelve 204
- AND la receta deja de aparecer en `GET /api/recipes`

#### Scenario: Receta inexistente

- GIVEN un id que no existe
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se devuelve 404
- AND no se borra ninguna otra receta

#### Scenario: Receta referenciada por el plan

- GIVEN una receta usada por una entrada del menú semanal
- WHEN se envía a `DELETE /api/recipes/{id}`
- THEN se devuelve 409 Conflict
- AND la receta y la entrada del plan permanecen sin cambios

#### Scenario: Borrado sin permiso

- GIVEN una petición sin credenciales válidas y el modo abierto desactivado
- WHEN se envía a `DELETE /api/recipes/{id}` sin el permiso `write:recipes`
- THEN se devuelve 401 si no hay sesión o 403 si falta el permiso
- AND la receta permanece sin cambios

### Requirement: Importación de receta desde URL

El sistema SHALL exponer `POST /api/recipes/import` (permiso `write:recipes`) con
cuerpo `{ "url": "<http(s)>" }`. SHALL descargar el HTML, extraer los bloques
`application/ld+json`, localizar el objeto `@type: Recipe` (también en `@graph`),
mapearlo a `RecipeInput`, crear la receta y responder `201` con la `Recipe`. SHALL
ser idempotente por `source_url`: si ya existe, no la duplica.

#### Scenario: Mapeo de campos JSON-LD

- GIVEN un bloque JSON-LD de `@type: Recipe`
- WHEN se importa la URL
- THEN `name` se mapea a nombre, `recipeIngredient` a ingredientes, `recipeYield` a `servings`, `prepTime`/`totalTime` a minutos, `recipeInstructions` a instrucciones y `url` a `source_url`

#### Scenario: Importación válida

- GIVEN una URL que devuelve HTML con un bloque JSON-LD de `@type: Recipe`
- WHEN se envía `POST /api/recipes/import` con esa URL
- THEN se crea la receta a partir de los campos JSON-LD
- AND se responde `201` con la receta creada y su `source_url`

#### Scenario: Recipe dentro de @graph

- GIVEN un HTML cuyo JSON-LD envuelve la receta en `@graph`
- WHEN se importa la URL
- THEN el sistema localiza el objeto `@type: Recipe` dentro del grafo
- AND mapea sus campos igual que si estuviera en la raíz

#### Scenario: Valores por defecto y conversión de tiempos

- GIVEN un JSON-LD `Recipe` sin `recipeYield` y con `prepTime: "PT1H20M"` y `totalTime: "PT1H45M"`
- WHEN se importa la URL
- THEN la receta creada tiene `servings` por defecto 2
- AND `prep_time_minutes` es 80 y el tiempo total es 105 minutos

#### Scenario: Instrucciones anidadas

- GIVEN un JSON-LD con `recipeInstructions` como lista de `HowToStep` y `HowToSection`
- WHEN se importa la URL
- THEN las instrucciones se aplanan a texto en el orden dado
- AND se persisten en el campo de instrucciones de la receta

#### Scenario: Importación idempotente por source_url

- GIVEN una receta ya almacenada con una `source_url` concreta
- WHEN se vuelve a importar la misma URL
- THEN no se crea una segunda receta
- AND se responde `200` con la receta existente

### Requirement: Protección contra SSRF

El sistema SHALL aplicar guardas de red antes y durante la descarga de la URL de
importación. SHALL aceptar únicamente los esquemas `http` y `https`; SHALL
resolver el host y RECHAZAR rangos privados, loopback, link-local y `0.0.0.0`
(tanto IPv4 como IPv6); SHALL limitar las redirecciones sin permitir saltos a
direcciones prohibidas; y SHALL imponer un timeout de conexión y lectura y un
tope de tamaño de descarga. Toda violación de estas guardas SHALL responder `400`.

#### Scenario: Esquema no permitido

- GIVEN una URL con esquema `file://` o `ftp://`
- WHEN se envía `POST /api/recipes/import`
- THEN se responde `400` sin realizar ninguna conexión

#### Scenario: Dirección privada o loopback

- GIVEN una URL cuyo host resuelve a `127.0.0.1`, `10.x.x.x`, `169.254.x.x` o `::1`
- WHEN se envía `POST /api/recipes/import`
- THEN se responde `400` sin descargar el contenido

#### Scenario: Redirección a destino prohibido

- GIVEN una URL pública que redirige a una dirección privada o loopback
- WHEN se sigue la redirección
- THEN el sistema rechaza el salto a la dirección prohibida
- AND se responde `400`

#### Scenario: Tiempo o tamaño fuera de límite

- GIVEN un origen que no responde dentro del timeout o cuya respuesta supera el tope de tamaño
- WHEN se importa la URL
- THEN la descarga se aborta
- AND se responde con el error correspondiente sin bloquear la petición indefinidamente

### Requirement: Errores de importación

El sistema SHALL distinguir los modos de fallo de la importación. Un HTML válido
pero sin JSON-LD de receta SHALL responder `422`; un fallo del origen
(timeout, error DNS o respuesta HTTP no-2xx) SHALL responder `502`; un
`Content-Type` que no sea HTML o una URL malformada SHALL responder `400`.

#### Scenario: HTML sin receta estructurada

- GIVEN una página HTML válida que no contiene ningún `@type: Recipe`
- WHEN se importa la URL
- THEN se responde `422` sin crear ninguna receta

#### Scenario: Fallo del origen

- GIVEN un origen que agota el timeout, falla la resolución DNS o responde con un estado no-2xx
- WHEN se importa la URL
- THEN se responde `502` sin crear ninguna receta

#### Scenario: Contenido no HTML

- GIVEN una respuesta con `Content-Type: application/json` o similar
- WHEN se importa la URL
- THEN se responde `400` sin intentar parsear el cuerpo

#### Scenario: URL malformada

- GIVEN una URL que no puede parsearse como URL absoluta válida
- WHEN se envía `POST /api/recipes/import`
- THEN se responde `400` sin intentar conectar

### Requirement: Desglose de cantidad y unidad al importar
Al importar, SHALL interpretar cada `recipeIngredient` para separar cantidad, unidad y nombre: SHALL reconocer una cantidad inicial (entero, decimal o fracción) y una unidad conocida por sus sinónimos normalizados sin conversión de magnitud (masa, volumen, recuento, `taza`, `cucharada`, `cucharadita`, `pizca`), guardando `amount` numérico, `unit` normalizada y un `name` sin la cantidad; sin cantidad detectable, SHALL usar `amount = 1`, `unit` "ud" y el nombre completo.

#### Scenario: Cantidad con unidad
- GIVEN un `recipeIngredient` «40 g de queso parmesano»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «queso parmesano», `amount` 40 y `unit` «g»

#### Scenario: Cantidad sin unidad
- GIVEN un `recipeIngredient` «1 calabacín»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «calabacín», `amount` 1 y `unit` «ud»

#### Scenario: Fracción
- GIVEN un `recipeIngredient` «1/2 taza de harina»
- WHEN se importa la receta
- THEN el ingrediente queda con `amount` 0.5, `unit` «taza» y `name` «harina»

#### Scenario: Sin cantidad detectable
- GIVEN un `recipeIngredient` «sal al gusto»
- WHEN se importa la receta
- THEN el ingrediente queda con `amount` 1, `unit` «ud» y `name` «sal al gusto»

#### Scenario: Sinónimo de masa en forma larga
- GIVEN un `recipeIngredient` «150 gramos de pechuga de pollo»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «pechuga de pollo», `amount` 150 y `unit` «g»

#### Scenario: Sinónimo de volumen
- GIVEN un `recipeIngredient` «2 litros de leche»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «leche», `amount` 2 y `unit` «l»

#### Scenario: Recuento en forma larga
- GIVEN un `recipeIngredient` «3 unidades de huevo»
- WHEN se importa la receta
- THEN el ingrediente queda con `name` «huevo», `amount` 3 y `unit` «ud»
