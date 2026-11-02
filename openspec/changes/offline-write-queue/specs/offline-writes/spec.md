# Capability: offline-writes

## Purpose
Define la escritura sin conexión en OxiMeal: las mutaciones de la lista de la compra se encolan en IndexedDB, se aplican de forma optimista en la caché local y se sincronizan con el servidor al recuperar la conexión, sin duplicar ítems gracias a un identificador de cliente idempotente.

## ADDED Requirements

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
