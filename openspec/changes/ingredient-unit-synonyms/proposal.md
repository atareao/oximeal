# Proposal

## Why

Al importar una receta, las líneas que usan el nombre completo de la unidad («150 gramos de pechuga de pollo», «2 litros de leche») no se desglosan: `parse_ingredient` solo reconoce abreviaturas (`g`, `kg`, `ml`, `l`), de modo que la unidad queda como `ud` y el nombre arrastra el texto de la unidad. Es el caso mayoritario en recetas reales y hace que el escalado por raciones y la consolidación de la lista de la compra no funcionen.

## What Changes

- Ampliar la lista blanca de unidades de `ingredient_parse` con sinónimos normalizados (singular/plural y formas largas) **sin conversión de magnitudes**: `gramo(s)`→`g`, `kilogramo(s)`/`kilo(s)`→`kg`, `mililitro(s)`→`ml`, `litro(s)`/`lt`→`l`, `unidad(es)`/`u`→`ud`, `cda`, `cdta`, `docena(s)`, etc.

## Capabilities

### New Capabilities
<!-- Ninguna. -->

### Modified Capabilities
- `recipes`: el requerimiento «Desglose de cantidad y unidad al importar» reconoce sinónimos de unidades (formas largas y plurales), no solo abreviaturas.

## Impact

- Backend: `backend/src/domain/ingredient_parse.rs` (lista blanca `KNOWN_UNITS` + tests). Sin cambios de API ni de esquema.
- Operación: la imagen `ghcr.io/atareao/oximeal:local` debe reconstruirse y el contenedor recrearse para que el cambio llegue al usuario.
