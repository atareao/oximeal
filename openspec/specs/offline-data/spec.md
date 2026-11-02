# offline-data Specification

## Purpose
Define la disponibilidad de datos sin conexión en OxiMeal: la persistencia de las respuestas de las consultas en IndexedDB, su hidratación al arrancar, el modo degradado que mantiene la interfaz con datos cacheados cuando la sesión ha caducado o no hay red, y la limpieza de esa caché al cerrar sesión.

## Requirements

### Requirement: Caché local de datos
La aplicación SHALL persistir en IndexedDB las respuestas de las consultas de datos (lista de la compra, plan del menú y recetas) y SHALL hidratar la caché de consultas desde IndexedDB al arrancar, de modo que los datos estén disponibles sin conexión.

#### Scenario: Arranque sin conexión con caché previa
- GIVEN una sesión anterior con conexión que dejó datos en IndexedDB
- WHEN la aplicación arranca sin conexión
- THEN la vista de la compra muestra los ítems guardados

#### Scenario: Persistencia tras una carga correcta
- GIVEN una carga de la lista de la compra correcta
- WHEN la petición termina
- THEN el resultado se guarda en IndexedDB

### Requirement: Modo degradado por sesión caducada o falta de red
Cuando no haya conexión y existan datos en caché, la aplicación SHALL mostrar esos datos con un aviso visible de modo degradado, en lugar de vaciar la interfaz o redirigir al login.

#### Scenario: Sin conexión con datos cacheados
- GIVEN la aplicación sin conexión y con datos en caché
- WHEN el usuario abre la vista de la compra
- THEN los datos cacheados se muestran con un aviso de modo degradado
- AND la aplicación no vacía la vista

#### Scenario: Recuperación de la conexión
- GIVEN la aplicación en modo degradado
- WHEN se recupera la conexión
- THEN el aviso desaparece y los datos se refrescan desde el servidor

### Requirement: Limpieza de la caché local al cerrar sesión
La aplicación SHALL eliminar la caché de datos de IndexedDB al cerrar sesión, para no exponer datos de un usuario a otro.

#### Scenario: Cierre de sesión limpia la caché
- GIVEN una sesión con datos cacheados
- WHEN el usuario cierra sesión
- THEN la caché de datos de IndexedDB se elimina
