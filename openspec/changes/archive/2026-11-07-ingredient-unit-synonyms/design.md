# Design

## Context

`ingredient_parse::parse_ingredient` reconoce la unidad con una lista blanca (`KNOWN_UNITS`) que hoy solo tiene abreviaturas y algunos plurales (`g`,`gr`,`kg`,`ml`,`l`,`cl`,`ud`,`uds`,`taza(s)`,`cucharada(s)`,`cucharadita(s)`,`pizca(s)`). `units.rs::canonicalize` conoce muchos más sinónimos, pero **convierte magnitudes** (kg→g·1000, l→ml·1000, cucharadita→cda/3), lo que no queremos al persistir el ingrediente: aquí se conserva la cantidad tal cual.

## Goals / Non-Goals

**Goals:**
- Reconocer formas largas y plurales comunes (gramos, kilogramos, mililitros, litros, unidades, cda, cdta, docenas) normalizadas a una unidad de presentación, sin convertir magnitudes.

**Non-Goals:**
- Conversión de magnitudes o familias (g↔kg, cucharadita↔cucharada) al importar.
- Refactorizar `units.rs` a una tabla única compartida (se deja como follow-up).

## Decisions

- **Extender `KNOWN_UNITS`** con los sinónimos → unidad normalizada (`gramos`→`g`, `kilos`→`kg`, `mililitros`→`ml`, `litros`/`lt`→`l`, `unidad`/`unidades`/`u`→`ud`, `cda`→`cucharada`, `cdta`→`cucharadita`, `docena(s)`→`docena`). Alternativa descartada por ahora: reutilizar `units::canonicalize` (convierte magnitudes) o unificar ambas listas en `units.rs` (mayor refactor).
- Mantener el resto de reglas intactas (conector «de», fracción/mixta, decimal coma/punto, fallback `1 ud`).

## Risks / Trade-offs

- **Duplicación parcial** de sinónimos con `units.rs`: se documenta como follow-up (tabla única alias→familia→presentación).
- **Falsos positivos**: acotado a una lista blanca explícita; una palabra no listada nunca se toma como unidad.
