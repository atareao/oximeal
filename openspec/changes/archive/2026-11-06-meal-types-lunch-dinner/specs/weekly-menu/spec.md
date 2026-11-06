# Spec Delta

## MODIFIED Requirements

### Requirement: Alta de receta en un slot

Desde la vista semanal, el usuario SHALL poder añadir una receta a un slot. La
acción SHALL abrir un selector que permita **buscar recetas** reutilizando el
catálogo (`useRecipes`), elegir el **tipo de comida** cuando la vista no lo fije
por slot, eligiendo únicamente entre **COMIDA y CENA**, elegir el **rol**
(`main|salad|side`) y las **raciones** (por defecto `2`, mínimo `1`); al
confirmar, la entrada SHALL crearse mediante `POST /api/meal-plan` y reflejarse
en el calendario.

#### Scenario: Añadir una receta a un slot
- GIVEN la vista semanal de una semana con conexión
- WHEN el usuario abre el selector de un slot, busca una receta, elige rol y raciones y confirma
- THEN se crea la entrada con `POST /api/meal-plan`
- AND la entrada aparece en el slot correspondiente tras invalidar la consulta del plan

#### Scenario: Búsqueda en el selector
- GIVEN el selector de recetas abierto
- WHEN el usuario escribe un término de búsqueda
- THEN el listado se filtra con la búsqueda del recetario
- AND se puede seleccionar una receta de los resultados

#### Scenario: Tipo de comida
- GIVEN un slot correspondiente a una comida concreta
- WHEN el usuario añade una receta desde ese slot
- THEN la entrada se crea con el `meal_type` del slot
- AND no se exige elegir el tipo de comida de nuevo

#### Scenario: Opciones de tipo de comida limitadas a COMIDA y CENA
- GIVEN el selector de tipo de comida al planificar una receta
- WHEN el usuario despliega las opciones disponibles
- THEN solo se ofrecen COMIDA y CENA
- AND no se ofrecen desayuno ni merienda

### Requirement: Vista diaria

La vista diaria SHALL mostrar un único día con las franjas COMIDA y CENA y las
mismas acciones de planificación que la vista semanal. SHALL permitir navegar al
día anterior, al siguiente y volver a hoy.

#### Scenario: Día con comidas

- GIVEN la vista diaria de un día con comidas
- WHEN se carga
- THEN se muestran las franjas COMIDA y CENA del día con sus recetas
- AND no se muestran franjas de desayuno ni de merienda

#### Scenario: Navegación entre días

- GIVEN la vista diaria
- WHEN el usuario pulsa día siguiente
- THEN se carga el día siguiente
- AND la URL refleja la nueva fecha

#### Scenario: Volver a hoy

- GIVEN la vista diaria de otra fecha
- WHEN el usuario pulsa Hoy
- THEN se carga la fecha actual

### Requirement: Vista semanal

La aplicación SHALL mostrar un calendario semanal de lunes a domingo con una tarjeta por día que incluya las franjas COMIDA y CENA, el nombre de la receta asignada y un marcador cuando la receta es una ensalada. La vista semanal SHALL ser la vista por defecto del menú y SHALL permitir navegar entre semanas y planificar desde la propia vista.

#### Scenario: Semana con datos

- GIVEN un plan semanal con recetas asignadas a comida y cena
- WHEN se abre la vista semanal
- THEN se muestran siete tarjetas, de lunes a domingo
- AND cada tarjeta muestra el nombre de la receta en COMIDA y CENA
- AND las recetas de categoría ensalada muestran el marcador de ensalada
- AND cada entrada planificada muestra las acciones de editar raciones y quitar

#### Scenario: Navegación entre semanas

- GIVEN la vista semanal abierta
- WHEN el usuario pulsa la acción de semana siguiente o anterior
- THEN la vista carga el plan de la semana seleccionada

#### Scenario: Franjas de la semana limitadas a COMIDA y CENA

- GIVEN la vista semanal
- WHEN se carga la semana
- THEN cada tarjeta muestra únicamente las franjas COMIDA y CENA
- AND no se muestran ni se ofrecen desayuno ni merienda

### Requirement: Vista mensual

La vista mensual SHALL mostrar una rejilla de calendario del mes con una celda por día. Cada celda SHALL mostrar el número de día y un resumen de las comidas planificadas. Al tocar una celda SHALL abrirse la vista diaria de esa fecha. SHALL permitir navegar al mes anterior y al siguiente.

#### Scenario: Rejilla del mes

- GIVEN un mes con comidas planificadas
- WHEN se abre la vista mensual
- THEN se muestra la rejilla del mes
- AND cada día muestra su resumen

#### Scenario: Abrir un día

- GIVEN la rejilla mensual
- WHEN el usuario toca un día
- THEN se abre la vista diaria de esa fecha

#### Scenario: Navegación entre meses

- GIVEN la vista mensual
- WHEN el usuario pulsa mes siguiente
- THEN se carga el mes siguiente

#### Scenario: Objetivo táctil

- GIVEN un viewport móvil
- WHEN se inspecciona una celda de día
- THEN su objetivo táctil es de al menos 44px

#### Scenario: Resumen mensual limitado a COMIDA y CENA

- GIVEN un mes con comidas planificadas
- WHEN se abre la vista mensual
- THEN el resumen de cada celda considera únicamente las franjas COMIDA y CENA
- AND no incluye desayuno ni merienda
