# Capability: auth-ui

## MODIFIED Requirements

### Requirement: Manejo de 401/403
Cuando cualquier petición devuelva 401, la aplicación SHALL llevar al usuario de nuevo al login, salvo que no haya conexión y existan datos cacheados, en cuyo caso SHALL mantener la sesión en modo degradado mostrando los datos guardados con un aviso. Cuando devuelva 403, SHALL mostrar un mensaje de permisos insuficientes **sin** destruir la sesión.

#### Scenario: 401 durante el uso
- GIVEN una aplicación en uso cuya sesión caduca
- WHEN una petición devuelve 401
- THEN la aplicación redirige de nuevo al login

#### Scenario: 401 sin conexión con datos en caché
- GIVEN una sesión caducada, sin conexión y con datos en caché
- WHEN la aplicación intenta consultar datos
- THEN los datos cacheados se muestran con un aviso de modo degradado
- AND la aplicación no redirige al login

#### Scenario: 403 durante el uso
- GIVEN una sesión válida sin permiso para una acción
- WHEN una petición devuelve 403
- THEN la aplicación muestra un aviso de permisos insuficientes y mantiene la sesión intacta
