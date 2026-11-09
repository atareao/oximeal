# weekly-menu Specification

## Purpose
Describe la vista del menú semanal de lunes a domingo, con una tarjeta por día y las franjas COMIDA y CENA, el marcador de ensalada y la navegación entre semanas. La vista obtiene los datos de `GET /api/meal-plan?start_date&end_date` y representa los estados de carga, error y vacío.

## Requirements

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

### Requirement: Baja con confirmación

La vista semanal SHALL permitir quitar una entrada del plan solicitando
**confirmación explícita** antes de eliminarla. Al confirmar, la entrada SHALL
eliminarse mediante `DELETE /api/meal-plan/{id}` y desaparecer del calendario.

#### Scenario: Quitar una entrada
- GIVEN una entrada planificada en el menú
- WHEN el usuario pulsa quitar y confirma en el diálogo
- THEN se llama a `DELETE /api/meal-plan/{id}`
- AND la entrada desaparece del slot tras invalidar la consulta del plan

#### Scenario: Cancelar la baja
- GIVEN el diálogo de confirmación de baja abierto
- WHEN el usuario cancela
- THEN no se llama a `DELETE /api/meal-plan/{id}`
- AND la entrada permanece en el menú

### Requirement: Edición de raciones desde el menú

La vista semanal SHALL permitir editar las raciones de una entrada existente y
persistirlas mediante `PATCH /api/meal-plan/{id}`, reflejando el nuevo valor sin
recrear la entrada.

#### Scenario: Editar raciones
- GIVEN una entrada planificada con `servings` igual a `2`
- WHEN el usuario cambia las raciones a `4` y confirma
- THEN se llama a `PATCH /api/meal-plan/{id}` con `servings` igual a `4`
- AND la entrada conserva su receta y su slot y muestra el nuevo valor

### Requirement: Validación y feedback de la planificación

La planificación SHALL validar en cliente los campos obligatorios y sus rangos
(receta seleccionada, `servings >= 1`) y SHALL mostrar feedback de **carga** y de
**error**, sin dejar el calendario en un estado inconsistente.

#### Scenario: Carga durante el alta
- GIVEN una receta seleccionada en el selector
- WHEN el usuario confirma la creación y la petición está en curso
- THEN la acción muestra un estado de carga y se evita el doble envío

#### Scenario: Error de validación
- GIVEN el selector de recetas sin receta seleccionada o con raciones menores que `1`
- WHEN el usuario intenta confirmar
- THEN se muestra un mensaje de validación
- AND no se envía ninguna petición al servidor

#### Scenario: Error del servidor
- GIVEN una petición de alta, baja o edición válida
- WHEN el servidor responde con error
- THEN se muestra un aviso de error
- AND el calendario mantiene el estado previo a la operación

#### Scenario: Error de carga del selector
- GIVEN que la consulta del catálogo de recetas falla al abrir el selector
- WHEN el usuario abre el selector
- THEN se muestra un estado de error con acción de reintento

### Requirement: Accesibilidad táctil de la planificación

Los controles de planificación (añadir, quitar, editar raciones y selector)
SHALL tener objetivos táctiles de al menos 48px, SHALL ser operables por teclado,
SHALL gestionar el foco al abrir y cerrar selector y diálogos, y SHALL exponer
etiquetas y roles accesibles.

#### Scenario: Objetivos táctiles
- GIVEN la vista semanal en un viewport móvil
- WHEN se inspeccionan los controles de añadir, quitar y editar raciones
- THEN su altura es de al menos 48px

#### Scenario: Foco en el selector
- GIVEN el selector de recetas cerrado
- WHEN el usuario lo abre
- THEN el foco se mueve al interior del selector
- AND al cerrarlo el foco vuelve al control que lo abrió

#### Scenario: Anuncio de acciones
- GIVEN una entrada planificada
- WHEN se recorre con un lector de pantalla
- THEN las acciones de quitar y editar raciones tienen nombre accesible que identifica la receta y la acción

### Requirement: Convivencia con Valet

La planificación nativa SHALL convivir con el asistente Valet: Valet SHALL
seguir disponible como vía alternativa y el estado vacío SHALL mantener su
acceso, sin que la nueva planificación lo retire.

#### Scenario: Valet sigue disponible
- GIVEN `VITE_VALET_URL` configurada
- WHEN el usuario abre la vista semanal
- THEN la acción para planificar con Valet sigue accesible

#### Scenario: Estado vacío con ambas vías
- GIVEN una semana sin entradas en el plan
- WHEN se carga la vista semanal
- THEN el estado vacío ofrece planificar una comida en la propia interfaz
- AND el estado vacío mantiene la acción de añadir comidas con Valet

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

### Requirement: Enlace a la receta en el menú

La vista diaria y la vista semanal SHALL mostrar el nombre de cada receta planificada como un enlace a su detalle en `/recipes/:id`.

#### Scenario: Enlace en la vista diaria
- **WHEN** se abre la vista diaria con una receta planificada
- **THEN** el nombre de la receta se muestra como enlace a `/recipes/:id`
- **AND** al pulsarlo se navega al detalle de la receta

#### Scenario: Enlace en la vista semanal
- **WHEN** se abre la vista semanal con recetas planificadas
- **THEN** cada nombre de receta se muestra como enlace a `/recipes/:id`
- **AND** al pulsarlo se navega al detalle de la receta

### Requirement: Añadir a la lista desde el menú

La vista diaria y la vista semanal SHALL ofrecer un botón «Añadir a la lista» junto a la fecha. En la vista diaria, el botón SHALL añadir los ingredientes del día visible. En la vista semanal, el botón de la cabecera SHALL añadir los ingredientes de la semana completa y cada tarjeta de día SHALL tener un botón que añada solo los ingredientes de ese día. La acción SHALL usar `POST /api/shopping-list/from-plan` con `{start_date, end_date}`. La acción SHALL estar deshabilitada sin conexión y SHALL mostrar el estado de carga y un error si falla.

#### Scenario: Añadir el día desde la vista diaria
- **GIVEN** la vista diaria de un día con recetas
- **WHEN** el usuario pulsa «Añadir a la lista» junto a la fecha
- **THEN** se llama a `POST /api/shopping-list/from-plan` con `start_date` y `end_date` iguales a ese día
- **AND** la lista de la compra se invalida y se refresca

#### Scenario: Añadir la semana desde la vista semanal
- **GIVEN** la vista semanal de una semana con recetas
- **WHEN** el usuario pulsa «Añadir a la lista» junto a la fecha
- **THEN** se llama a `POST /api/shopping-list/from-plan` con `start_date` el lunes y `end_date` el domingo
- **AND** la lista de la compra se invalida y se refresca

#### Scenario: Añadir un día desde la vista semanal
- **GIVEN** la vista semanal con una tarjeta de día
- **WHEN** el usuario pulsa el botón «Añadir a la lista» de esa tarjeta
- **THEN** se llama a `POST /api/shopping-list/from-plan` con `start_date` y `end_date` iguales a ese día
- **AND** la lista de la compra se invalida y se refresca

#### Scenario: Sin conexión
- **GIVEN** la vista sin conexión
- **WHEN** el usuario intenta añadir a la lista
- **THEN** el botón está deshabilitado
- **AND** no se realiza la petición

#### Scenario: Error de la petición
- **GIVEN** la vista con conexión
- **WHEN** la petición de añadir a la lista falla
- **THEN** se muestra un mensaje de error
- **AND** la lista no se modifica
