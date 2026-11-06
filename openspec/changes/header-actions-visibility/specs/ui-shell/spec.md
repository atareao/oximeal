# Capability: ui-shell

## MODIFIED Requirements

### Requirement: Shell responsive

La aplicación SHALL usar una única barra de navegación superior fija en todos los
tamaños de pantalla. SHALL eliminar el panel lateral (`Layout.Sider`) y la
navegación inferior (`BottomNav`). La barra SHALL mostrar el logo de OxiMeal a la
izquierda, las entradas Menú, Recetas y Compra, y a la derecha Configuración y
Salir. En viewports de ancho inferior a 480px el logo SHALL renderizarse en forma
compacta, sin el texto «OxiMeal» y conservando el nombre accesible, y los huecos y
padding SHALL reducirse para que Configuración y Salir permanezcan dentro del
viewport, sin recorte ni desplazamiento horizontal. El grupo de acciones SHALL
ser no compresible (`flex-shrink: 0`) y todos los controles SHALL conservar una
altura mínima de 48px.

#### Scenario: Barra única

- GIVEN cualquier viewport
- WHEN se carga la aplicación
- THEN se muestra una única barra superior fija
- AND no existe panel lateral ni navegación inferior

#### Scenario: Navegación en desktop

- GIVEN un viewport de ancho igual o superior al breakpoint desktop
- WHEN se carga la aplicación
- THEN la barra superior muestra las entradas con icono y texto
- AND no se muestra la navegación inferior fija

#### Scenario: Navegación en móvil

- GIVEN un viewport de ancho inferior al breakpoint desktop
- WHEN se carga la aplicación
- THEN la barra superior muestra las entradas solo con icono
- AND no se muestra el panel lateral

#### Scenario: Barra en móvil

- GIVEN un viewport inferior a 768px
- WHEN se renderiza la barra superior
- THEN cada entrada muestra solo un icono con nombre accesible
- AND cada control tiene una altura mínima de 48px

#### Scenario: Barra en desktop

- GIVEN un viewport de 768px o más
- WHEN se renderiza la barra superior
- THEN cada entrada muestra icono y texto

#### Scenario: Barra en móvil estrecha

- GIVEN un viewport de 360px de ancho
- WHEN se renderiza la barra superior
- THEN el logo muestra la forma compacta con nombre accesible «OxiMeal»
- AND Configuración y Salir son visibles y están dentro del viewport
- AND el grupo de acciones no se comprime
