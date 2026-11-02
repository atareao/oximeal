# Capability: shopping-mode

## MODIFIED Requirements

### Requirement: Marcar como comprado
Al tocar un ítem, la aplicación SHALL marcarlo como comprado aplicando tachado y opacidad del 40%, SHALL moverlo al final de su categoría y SHALL emitir feedback táctil mediante la Vibration API (`navigator.vibrate`) cuando esté disponible. En navegadores sin soporte (p. ej. iOS Safari) la llamada SHALL ser un no-op.

#### Scenario: Marcar un ítem
- GIVEN un ítem pendiente dentro de una categoría
- WHEN el usuario lo toca
- THEN el ítem se muestra tachado con opacidad del 40%
- AND se mueve al final de su categoría
- AND se emite una vibración si `navigator.vibrate` está disponible

#### Scenario: Desmarcar un ítem
- GIVEN un ítem marcado como comprado
- WHEN el usuario lo vuelve a tocar
- THEN el ítem recupera su estilo normal y su posición pendiente

#### Scenario: Navegador sin Vibration API
- GIVEN un navegador sin `navigator.vibrate` (p. ej. iOS Safari)
- WHEN el usuario marca un ítem
- THEN la aplicación no lanza ningún error y el ítem se marca igualmente
