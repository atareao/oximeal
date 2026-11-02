# recipes Specification

## Purpose
Esta capacidad cubre la gestión de recetas: creación, consulta, actualización y borrado (CRUD), así como el listado y la búsqueda de recetas. La búsqueda combina SQLite FTS5 con un fallback por LIKE y permite filtrar por categorías.

## Requirements

### Requirement: Listado y búsqueda de recetas
El sistema SHALL exponer `GET /api/recipes` con filtro de texto opcional `q` (FTS5 trigram) y filtro `category`, con paginación.

#### Scenario: Búsqueda por substring
- GIVEN recetas que contienen "tomate" en nombre o instrucciones
- WHEN se consulta `GET /api/recipes?q=tomate`
- THEN se devuelven las recetas coincidentes con metadatos de paginación

#### Scenario: Query corta
- GIVEN una query de menos de 3 caracteres
- WHEN se consulta el listado
- THEN el sistema usa fallback LIKE en lugar de MATCH

### Requirement: Alta de receta
El sistema SHALL exponer `POST /api/recipes` que valida categoría (main|salad|side|dessert|breakfast) e ingredientes, y persiste la receta.

#### Scenario: Creación válida
- GIVEN un payload de receta válido
- WHEN se envía a `POST /api/recipes`
- THEN se crea la receta con id y se devuelve 201

#### Scenario: Categoría inválida
- GIVEN un payload con categoría no permitida
- WHEN se envía
- THEN se devuelve 400 y no se persiste nada

### Requirement: Detalle de receta
El sistema SHALL exponer `GET /api/recipes/{id}` devolviendo la receta completa con ingredientes.

#### Scenario: No encontrada
- GIVEN un id inexistente
- WHEN se consulta el detalle
- THEN se devuelve 404

### Requirement: Actualización de receta

El sistema SHALL exponer `PUT /api/recipes/{id}` con permiso `write:recipes`, que
reemplaza la receta completa —nombre, categoría, `diet_type`,
`prep_time_minutes`, `servings`, ingredientes, instrucciones y `source_url`—,
responde `200` con la `Recipe` actualizada y mantiene sincronizado el índice de
búsqueda FTS.

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
