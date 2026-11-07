# Capability: recipes

## MODIFIED Requirements

### Requirement: Detalle de receta

El sistema SHALL exponer `GET /api/recipes/{id}` devolviendo la receta completa
con ingredientes y etiquetas, cada etiqueta con `id`, `name`, `slug` y `source`.
El detalle SHALL mostrar las cantidades escaladas a las raciones seleccionadas
(1..20), con la preferencia de comensales como valor inicial, aviso del cálculo y
una opción para ver las cantidades originales. Si la receta no tiene cantidades
reales (todas `amount = 1` y `unit = "ud"`, caso de importación), SHALL mostrar
las originales con aviso y no escalar.

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
