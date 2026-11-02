# Capability: persistence

## Purpose

Esta capacidad define el comportamiento de escritura transaccional y las garantías de integridad del esquema SQLite de OxiMeal: transacciones de escritura con `BEGIN IMMEDIATE`, claves foráneas explícitas y restricciones de dominio `CHECK`.

## ADDED Requirements

### Requirement: Transacciones de escritura en modo IMMEDIATE
El sistema SHALL abrir con `BEGIN IMMEDIATE` toda transacción que vaya a escribir, de forma que el bloqueo de escritura se adquiera al inicio y un patrón *read-then-write* concurrente NO pueda fallar con `SQLITE_BUSY_SNAPSHOT`.

#### Scenario: Escrituras concurrentes sin BUSY_SNAPSHOT
- GIVEN un pool SQLite en modo WAL con dos o más conexiones y `busy_timeout` configurado
- WHEN dos o más tareas ejecutan a la vez una escritura transaccional (`add_shopping_items`, `consolidate` o `set_weekly_plan`)
- THEN todas terminan correctamente y ninguna falla con `SQLITE_BUSY_SNAPSHOT`

#### Scenario: Reproducción del defecto con BEGIN diferido
- GIVEN dos conexiones, la primera con `BEGIN` diferido que ha leído (snapshot) y la segunda que escribe y confirma
- WHEN la primera conexión intenta escribir
- THEN la operación falla con `SQLITE_BUSY_SNAPSHOT`, documentando el defecto que `BEGIN IMMEDIATE` evita

### Requirement: Clave foránea explícita con ON DELETE
El esquema SHALL declarar explícitamente `ON DELETE RESTRICT` en la clave foránea `meal_plan_entries.recipe_id → recipes(id)`, de modo que borrar una receta referenciada por el plan falle y NO deje entradas huérfanas.

#### Scenario: Borrado de receta referenciada
- GIVEN una receta referenciada por al menos una entrada de `meal_plan_entries`
- WHEN se intenta `DELETE FROM recipes` para esa receta
- THEN la operación falla por violación de clave foránea y la entrada del plan permanece

### Requirement: Restricciones CHECK de dominio
El esquema SHALL aplicar `CHECK` como defensa en profundidad: `shopping_list_items.amount >= 0`, `recipes.servings` y `meal_plan_entries.servings` NULL o `>= 1`, y `recipes.prep_time_minutes` NULL o `>= 0`. Un valor inválido SHALL ser rechazado por la base de datos.

#### Scenario: Cantidad negativa rechazada
- GIVEN un intento de insertar o actualizar `shopping_list_items.amount < 0`
- WHEN la base de datos evalúa el `CHECK`
- THEN la operación falla por violación de restricción

#### Scenario: Raciones menores que uno rechazadas
- GIVEN un intento de persistir `servings < 1` en `recipes` o `meal_plan_entries`
- WHEN la base de datos evalúa el `CHECK`
- THEN la operación falla por violación de restricción

#### Scenario: Tiempo de preparación negativo rechazado
- GIVEN un intento de persistir `prep_time_minutes < 0` en `recipes`
- WHEN la base de datos evalúa el `CHECK`
- THEN la operación falla por violación de restricción

#### Scenario: Valores válidos y NULL aceptados
- GIVEN `servings = 2`, `prep_time_minutes = 15` y `amount = 0`
- WHEN se persisten en sus tablas
- THEN la operación se completa y, además, `servings` y `prep_time_minutes` admiten NULL
