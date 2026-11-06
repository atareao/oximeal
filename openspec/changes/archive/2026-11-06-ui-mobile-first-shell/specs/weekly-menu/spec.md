## MODIFIED Requirements

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

### Requirement: Datos desde la API

La vista de menú SHALL obtener los datos de `GET /api/meal-plan?start_date&end_date` con el rango correspondiente a la vista activa: el día en la vista diaria, la semana de lunes a domingo en la semanal y el mes visible en la mensual. SHALL representar los estados de carga, error y vacío.

#### Scenario: Semana vacía

- GIVEN una semana sin entradas en el plan
- WHEN se carga la vista semanal
- THEN se muestra un estado vacío con una acción para añadir comidas

#### Scenario: Rango diario

- GIVEN la vista diaria
- WHEN se carga
- THEN la petición cubre ese día

#### Scenario: Rango semanal

- GIVEN la vista semanal
- WHEN se carga
- THEN la petición cubre de lunes a domingo

#### Scenario: Rango mensual

- GIVEN la vista mensual
- WHEN se carga
- THEN la petición cubre el mes visible

#### Scenario: Error de red

- GIVEN que la petición a `GET /api/meal-plan` falla
- WHEN se carga la vista
- THEN se muestra un estado de error con una acción de reintento

## ADDED Requirements

### Requirement: Selector de vista

La vista de menú SHALL ofrecer tres vistas: Diaria, Semanal y Mensual. La vista por defecto SHALL ser Semanal. La vista activa SHALL reflejarse en el parámetro `view` de la URL (`day`, `week`, `month`) y SHALL conservarse al navegar entre periodos.

#### Scenario: Vista por defecto

- GIVEN el usuario abre `/` sin parámetro `view`
- WHEN se carga la página
- THEN se muestra la vista semanal

#### Scenario: Cambio de vista

- GIVEN la vista semanal
- WHEN el usuario elige Mensual
- THEN se muestra la rejilla mensual
- AND la URL incluye `view=month`

#### Scenario: Persistencia en la URL

- GIVEN la vista diaria de una fecha
- WHEN el usuario recarga la página
- THEN se mantiene la vista diaria y la fecha

### Requirement: Vista diaria

La vista diaria SHALL mostrar un único día con todas sus franjas (desayuno, comida, cena y tentempié) y las mismas acciones de planificación que la vista semanal. SHALL permitir navegar al día anterior, al siguiente y volver a hoy.

#### Scenario: Día con comidas

- GIVEN la vista diaria de un día con comidas
- WHEN se carga
- THEN se muestran todas las franjas del día con sus recetas

#### Scenario: Navegación entre días

- GIVEN la vista diaria
- WHEN el usuario pulsa día siguiente
- THEN se carga el día siguiente
- AND la URL refleja la nueva fecha

#### Scenario: Volver a hoy

- GIVEN la vista diaria de otra fecha
- WHEN el usuario pulsa Hoy
- THEN se carga la fecha actual

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
