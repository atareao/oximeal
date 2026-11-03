# offline-writes Specification

## Purpose
Define la escritura sin conexión en OxiMeal: las mutaciones de la lista de la compra se encolan en IndexedDB, se aplican de forma optimista en la caché local y se sincronizan con el servidor al recuperar la conexión, sin duplicar ítems gracias a un identificador de cliente idempotente.

## Requirements

### Requirement: Cola de escrituras offline
Cuando una mutación de la lista de la compra se realice sin conexión, la aplicación SHALL encolarla en IndexedDB y aplicarla de forma optimista sobre la caché local, mostrando el estado pendiente de sincronizar.

#### Scenario: Alta sin conexión
- GIVEN la lista de la compra abierta sin conexión
- WHEN el usuario añade un ítem
- THEN el ítem aparece en la lista local con estado pendiente
- AND la mutación se encola para sincronizarse

#### Scenario: Sincronización al recuperar la conexión
- GIVEN una cola con mutaciones pendientes
- WHEN la aplicación recupera la conexión
- THEN las mutaciones se envían al servidor en orden de encolado
- AND la lista se reconcilia con el estado del servidor

### Requirement: Reconciliación de identificadores y conflictos
La cola SHALL usar un identificador de cliente para el alta, SHALL reconciliar el identificador provisional con el real que devuelve el servidor tras el alta y SHALL resolver los conflictos de marcado por el criterio del último `updated_at` ganador.

#### Scenario: Toggle de un ítem aún no sincronizado
- GIVEN un ítem creado sin conexión con identificador provisional
- WHEN el usuario lo marca como comprado antes de sincronizar
- THEN el cambio se aplica a la operación encolada y se envía tras el alta

#### Scenario: Conflicto de marcado
- GIVEN un ítem modificado en el servidor y en la cola local
- WHEN la cola sincroniza la operación
- THEN prevalece el cambio con el `updated_at` más reciente

### Requirement: Tolerancia a fallos del drenado
La cola SHALL detenerse y reintentar cuando el drenado falle por falta de red o por 401, y SHALL descartar la operación notificándolo al usuario cuando falle por un 4xx no recuperable.

#### Scenario: Fallo de red durante el drenado
- GIVEN una cola drenándose sin conexión
- WHEN una operación falla por red
- THEN la cola se detiene y las operaciones pendientes se conservan

#### Scenario: Operación inválida
- GIVEN una operación encolada que el servidor rechaza con 4xx
- WHEN la cola la procesa
- THEN la operación se descarta y el usuario recibe un aviso

### Requirement: Reintento del drenado

La cola SHALL reintentar el drenado por sí sola cuando se detenga por un
fallo de red, con espera creciente y acotada, hasta que la conexión vuelva o la
cola quede vacía, sin intervención del usuario.

#### Scenario: Reintento hasta que vuelve la conexión

- GIVEN una cola con operaciones pendientes y un drenado que se detiene por fallo de red
- WHEN la conexión vuelve sin que cambie el estado del evento `online`
- THEN la cola reintenta el drenado sola
- AND las operaciones se sincronizan en orden y sin duplicados

#### Scenario: Espera creciente y acotada

- GIVEN un drenado que se detiene por red de forma repetida
- WHEN cada intento vuelve a fallar
- THEN el intervalo entre intentos crece hasta un máximo acotado
- AND no se usa un temporizador periódico

#### Scenario: Reinicio del backoff

- GIVEN un backoff en curso
- WHEN el drenado progresa o la cola queda vacía
- THEN el intervalo vuelve a su valor mínimo
- AND no se programa ningún reintento si no quedan operaciones

#### Scenario: Reintento al volver al primer plano

- GIVEN una cola con operaciones pendientes y un backoff en curso
- WHEN la pestaña vuelve al primer plano
- THEN se intenta el drenado de inmediato

#### Scenario: Sin reintento en bucle ante un 401

- GIVEN un drenado detenido por 401
- WHEN pasa el tiempo sin cambios en las condiciones
- THEN no se reintenta en bucle
- AND la cola se conserva para el siguiente cambio de conexión, visibilidad o cola

### Requirement: Planificación de comidas requiere conexión

Las mutaciones de la planificación de comidas (`POST`, `PATCH` y `DELETE` sobre
`/api/meal-plan`) SHALL requerir conexión y SHALL **no** encolarse en IndexedDB,
a diferencia de las escrituras de la lista de la compra. Sin conexión la
aplicación SHALL deshabilitar los controles de planificación y SHALL mostrar un
aviso de modo degradado, sin aplicar cambios optimistas que prometan una
persistencia inexistente.

#### Scenario: Planificación sin conexión
- GIVEN la vista semanal abierta sin conexión
- WHEN el usuario intenta añadir, editar raciones o quitar una entrada
- THEN los controles de planificación están deshabilitados
- AND se muestra un aviso de que la planificación requiere conexión
- AND no se encola ninguna operación en IndexedDB

#### Scenario: Recuperación de la conexión
- GIVEN la vista semanal en modo degradado sin conexión
- WHEN se recupera la conexión
- THEN los controles de planificación se habilitan de nuevo
- AND el plan se refresca desde el servidor
