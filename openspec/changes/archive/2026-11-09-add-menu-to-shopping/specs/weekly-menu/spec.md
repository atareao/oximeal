# Spec Delta

## ADDED Requirements

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

La vista diaria y la vista semanal SHALL ofrecer un botón «Añadir a la lista» junto a la fecha. En la diaria SHALL añadir los ingredientes del día visible; en la semanal, el de la cabecera SHALL añadir la semana completa y cada tarjeta de día SHALL tener otro que añada solo ese día. La acción SHALL usar `POST /api/shopping-list/from-plan` con `{start_date, end_date}`, SHALL estar deshabilitada sin conexión y SHALL mostrar carga y error si falla.

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