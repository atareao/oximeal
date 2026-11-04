# Capability: ui-shell

## ADDED Requirements

### Requirement: Pestaña Recetas

El shell de navegación SHALL incluir una entrada Recetas en `AppLayout` (Sider)
y en `BottomNav` (🍳 Recetas) que navega a `/recipes` y refleja el estado activo
cuando la ruta está seleccionada.

#### Scenario: Entrada en el Sider

- GIVEN un viewport desktop
- WHEN se carga la aplicación
- THEN el panel lateral muestra la sección Recetas
- AND al pulsarla se navega a `/recipes`

#### Scenario: Pestaña en la navegación inferior

- GIVEN un viewport móvil
- WHEN se carga la aplicación
- THEN la navegación inferior incluye la pestaña 🍳 Recetas
- AND al pulsarla se navega a `/recipes`

#### Scenario: Estado activo

- GIVEN la ruta `/recipes` activa
- WHEN se renderiza la navegación
- THEN la entrada Recetas aparece marcada como activa
- AND el resto de entradas no lo está
