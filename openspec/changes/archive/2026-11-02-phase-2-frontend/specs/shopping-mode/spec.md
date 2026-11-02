# Capability: shopping-mode

## ADDED Requirements

### Requirement: Agrupación por pasillo
La vista de la compra SHALL agrupar los ítems por pasillo, mostrando un encabezado con emoji para cada categoría: 🥦 Verdulería, 🐟 Pescadería, 🥩 Carnicería, ❄️ Refrigerados, 🍞 Panadería, 🥤 Bebidas, 🥫 Despensa y Otros.

#### Scenario: Ítems de varios pasillos
- GIVEN una lista con ítems de distintas categorías
- WHEN se abre el modo compra
- THEN los ítems aparecen agrupados bajo el encabezado de su pasillo
- AND cada encabezado incluye el emoji correspondiente

### Requirement: Marcar como comprado
Al tocar un ítem, la aplicación SHALL marcarlo como comprado aplicando tachado y opacidad del 40%, SHALL moverlo al final de su categoría y SHALL emitir una vibración mediante Web Haptics cuando esté disponible.

#### Scenario: Marcar un ítem
- GIVEN un ítem pendiente dentro de una categoría
- WHEN el usuario lo toca
- THEN el ítem se muestra tachado con opacidad del 40%
- AND se mueve al final de su categoría
- AND se emite una vibración si Web Haptics está disponible

#### Scenario: Desmarcar un ítem
- GIVEN un ítem marcado como comprado
- WHEN el usuario lo vuelve a tocar
- THEN el ítem recupera su estilo normal y su posición pendiente

### Requirement: Alta rápida y limpieza
La vista SHALL ofrecer un input superior para añadir ítems manualmente y una acción "Limpiar comprados" que invoca `DELETE /api/shopping-list/completed`.

#### Scenario: Alta manual
- GIVEN el modo compra abierto
- WHEN el usuario escribe un ítem en el input superior y confirma
- THEN el ítem se crea y aparece bajo el pasillo que le corresponde

#### Scenario: Limpiar comprados
- GIVEN una lista con ítems marcados como comprados
- WHEN el usuario pulsa "Limpiar comprados"
- THEN se invoca `DELETE /api/shopping-list/completed`
- AND los ítems comprados desaparecen de la vista
