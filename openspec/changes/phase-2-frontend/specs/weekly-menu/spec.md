# Capability: weekly-menu

## ADDED Requirements

### Requirement: Vista semanal
La aplicación SHALL mostrar un calendario semanal de lunes a domingo con una tarjeta por día que incluya las franjas COMIDA y CENA, el nombre de la receta asignada y un marcador cuando la receta es una ensalada. La vista SHALL permitir navegar entre semanas.

#### Scenario: Semana con datos
- GIVEN un plan semanal con recetas asignadas a comida y cena
- WHEN se abre la vista de menú semanal
- THEN se muestran siete tarjetas, de lunes a domingo
- AND cada tarjeta muestra el nombre de la receta en COMIDA y CENA
- AND las recetas de categoría ensalada muestran el marcador de ensalada

#### Scenario: Navegación entre semanas
- GIVEN la vista de menú semanal abierta
- WHEN el usuario pulsa la acción de semana siguiente o anterior
- THEN la vista carga el plan de la semana seleccionada

### Requirement: Datos desde la API
La vista semanal SHALL obtener los datos de `GET /api/meal-plan?start_date&end_date` y SHALL representar los estados de carga, error y vacío.

#### Scenario: Semana vacía
- GIVEN una semana sin entradas en el plan
- WHEN se carga la vista semanal
- THEN se muestra un estado vacío con una acción para añadir comidas

#### Scenario: Error de red
- GIVEN que la petición a `GET /api/meal-plan` falla
- WHEN se carga la vista semanal
- THEN se muestra un estado de error con una acción de reintento
- AND al pulsar reintentar se repite la petición
