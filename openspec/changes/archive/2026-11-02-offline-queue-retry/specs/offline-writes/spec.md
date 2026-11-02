# Capability: offline-writes

## ADDED Requirements

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
