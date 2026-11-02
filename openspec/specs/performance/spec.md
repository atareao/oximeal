# performance Specification

## Purpose

Esta capability fija los presupuestos de rendimiento del binario monolítico con
la UI embebida: RSS estable (≤16 MB) y pico (≤20 MB), FCP (≤1,5 s), transferencia
del grafo inicial declarado (≤500 kB) y objetivos táctiles ≥48 × 48 px. Su
verificación es una medición reproducible on-demand con `just perf`, **no un
*gate* de CI** (ADR-31): las métricas en CI son ruidosas y frágiles. La
metodología, los umbrales y los resultados se documentan en `docs/performance.md`.

## Requirements

### Requirement: Presupuesto de memoria

El binario compilado con `--release --features embed-ui` SHALL mantener, en
reposo y tras servir el shell, un **RSS estable ≤16 MB** y un **RSS pico ≤20 MB**
(el techo es el objetivo global de `PLAN.md` §10). La medición SHALL ser
reproducible con una receta `just` que registre el pico y el valor estable.

#### Scenario: Medición reproducible por debajo del techo

- GIVEN el binario compilado con `--release --features embed-ui`
- WHEN se ejecuta la receta `just` de medición de RSS sobre una base temporal
- THEN la receta arranca el servidor, sirve el shell y muestrea el RSS
- AND el RSS estable queda en ≤16 MB y el RSS pico en ≤20 MB
- AND la receta termina con código de salida 0

#### Scenario: Medición registrada con metodología

- GIVEN una ejecución de la receta de medición
- WHEN la receta produce sus resultados
- THEN quedan registrados el valor pico y el valor estable
- AND `docs/performance.md` documenta el método de muestreo, los umbrales y el criterio de las cifras (el margen de ~4 MB bajo el techo absorbe picos transitorios)

#### Scenario: Superación del presupuesto

- GIVEN una ejecución en la que el RSS supera los umbrales fijados
- WHEN la receta compara el resultado con el presupuesto
- THEN la receta falla de forma explícita indicando el valor medido

### Requirement: Presupuesto de rendimiento web

El shell servido por el binario SHALL cumplir presupuestos explícitos medidos
sobre `http://127.0.0.1` con Lighthouse y/o la Performance API: **primer
contenido útil (FCP) ≤1,5 s** y **peso de la transferencia inicial ≤500 kB**
(coherente con el presupuesto de bundle de 500 kB ya verificado por
`check-bundle`). La medición SHALL ser reproducible mediante un script.

#### Scenario: Medición de presupuestos web

- GIVEN el binario sirviendo el shell en `http://127.0.0.1`
- WHEN se ejecuta el script de auditoría de rendimiento con el navegador headless
- THEN se registran el tiempo hasta el primer contenido útil y el peso de la transferencia inicial
- AND ambos valores quedan dentro de los presupuestos fijados

#### Scenario: Incumplimiento del presupuesto web

- GIVEN una medición en la que el FCP o el peso inicial superan el presupuesto
- WHEN el script evalúa el resultado
- THEN el script falla de forma explícita señalando la métrica incumplida

### Requirement: Verificación táctil

Los controles principales de la UI mobile-first SHALL tener objetivos de
**al menos 48×48 px**, y las interacciones clave (navegación y modo compra)
SHALL responder correctamente **con y sin conexión**. La comprobación SHALL ser
automatizada o, en su defecto, documentada de forma verificable.

#### Scenario: Objetivos táctiles mínimos

- GIVEN la UI mobile-first servida por el binario
- WHEN se inspeccionan los controles principales (navegación y modo compra)
- THEN cada objetivo interactivo mide al menos 48×48 px

#### Scenario: Interacciones clave con conexión

- GIVEN el shell cargado con red disponible
- WHEN el usuario navega y usa el modo compra
- THEN las interacciones responden y completan su flujo esperado

#### Scenario: Interacciones clave sin conexión

- GIVEN el shell cargado y la red no disponible
- WHEN el usuario navega y usa el modo compra
- THEN la navegación sigue disponible desde la caché del Service Worker
- AND las escrituras se encolan para sincronizarse al recuperar la conexión
