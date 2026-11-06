# ui-shell Specification

## Purpose
Define el shell responsive mobile-first de la PWA de OxiMeal: en desktop muestra un panel lateral (`Layout.Sider`) y en móvil una navegación inferior con las pestañas Menú, Recetas, Compra y Asistente, con objetivos táctiles de al menos 48px, tema de Ant Design vía `ConfigProvider` y respeto de los safe-area insets. La pestaña Asistente abre o embebe Valet mediante `VITE_VALET_URL`, sin chat nativo.

## Requirements

### Requirement: Shell responsive

La aplicación SHALL usar una única barra de navegación superior fija en todos los tamaños de pantalla. SHALL eliminar el panel lateral (`Layout.Sider`) y la navegación inferior (`BottomNav`). La barra SHALL mostrar el logo de OxiMeal a la izquierda, las entradas Menú, Recetas y Compra, y a la derecha Configuración y Salir.

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

### Requirement: Objetivos táctiles

En móvil, las filas y controles interactivos SHALL tener una altura mínima de
48px para garantizar una interacción táctil cómoda. Esta regla SHALL aplicar
también a los controles de planificación del menú semanal (añadir receta, quitar
entrada, editar raciones y selector de recetas).

#### Scenario: Control interactivo en móvil
- GIVEN la vista de lista de la compra en un viewport móvil
- WHEN se inspecciona una fila o control interactivo
- THEN su altura es de al menos 48px

#### Scenario: Controles de planificación en móvil
- GIVEN la vista de menú semanal en un viewport móvil
- WHEN se inspeccionan los controles de añadir receta, quitar entrada y editar raciones
- THEN su altura es de al menos 48px

### Requirement: Tema Antd y safe-area

La aplicación SHALL configurar Ant Design mediante `ConfigProvider` con tokens dependientes del modo activo (claro u oscuro) y SHALL respetar los safe-area insets en la barra superior y en el contenido.

#### Scenario: Aplicación del tema

- GIVEN la aplicación cargada
- WHEN se renderiza el árbol de componentes
- THEN los componentes Antd heredan los tokens definidos en `ConfigProvider`
- AND el layout aplica los safe-area insets en los bordes superior e inferior

#### Scenario: Modo oscuro

- GIVEN el modo oscuro activo
- WHEN se renderiza el árbol de componentes
- THEN los componentes Antd usan `theme.darkAlgorithm`
- AND `color-scheme` es `dark`

#### Scenario: Modo claro

- GIVEN el modo claro activo
- WHEN se renderiza el árbol de componentes
- THEN los componentes Antd usan `theme.defaultAlgorithm`
- AND `color-scheme` es `light`

#### Scenario: Safe-area

- GIVEN un dispositivo con notch
- WHEN se muestra la barra superior
- THEN se aplica `env(safe-area-inset-top)`

### Requirement: Pestaña Asistente

La navegación superior SHALL NOT incluir una entrada Asistente. El acceso a Valet SHALL mantenerse desde las acciones de la vista de menú, usando `VITE_VALET_URL`, sin chat nativo.

#### Scenario: Apertura de Valet

- GIVEN `VITE_VALET_URL` configurada
- WHEN el usuario busca planificar con Valet desde el menú
- THEN se abre o embebe Valet en la URL configurada
- AND OxiMeal no muestra un chat propio

#### Scenario: Sin entrada Asistente

- GIVEN `VITE_VALET_URL` configurada
- WHEN se renderiza la barra superior
- THEN no aparece una entrada Asistente

#### Scenario: Valet desde el menú

- GIVEN la vista de menú
- WHEN el usuario busca planificar con Valet
- THEN la acción sigue disponible

### Requirement: Arranque sin antd en el critical path
El shell inicial SHALL renderizarse sin cargar antd de forma estática. Los proveedores y componentes de antd SHALL cargarse de forma diferida tras el primer pintado, y el grafo de módulos estáticos del entry SHALL mantenerse por debajo de 500 kB en bruto.

#### Scenario: Primer pintado ligero
- GIVEN una carga en frío de la aplicación
- WHEN el navegador ejecuta el entry
- THEN el shell muestra un fallback ligero sin haber descargado el chunk de antd
- AND el chunk de antd se solicita de forma asíncrona

#### Scenario: Presupuesto del grafo estático
- GIVEN el build de producción
- WHEN se ejecuta la comprobación de presupuesto
- THEN la suma de los chunks estáticos del entry es menor de 500 kB en bruto

#### Scenario: Aviso de 403 sin import estático
- GIVEN una respuesta 403 sin handler registrado
- WHEN el cliente procesa la respuesta
- THEN el aviso se muestra cargando antd de forma dinámica

### Requirement: Pestaña Recetas

El shell SHALL incluir una entrada Recetas en la barra superior que navega a `/recipes` y refleja el estado activo cuando la ruta está seleccionada.

#### Scenario: Entrada en el Sider

- GIVEN un viewport desktop
- WHEN se carga la aplicación
- THEN la barra superior muestra la entrada Recetas
- AND al pulsarla se navega a `/recipes`

#### Scenario: Pestaña en la navegación inferior

- GIVEN un viewport móvil
- WHEN se carga la aplicación
- THEN la barra superior incluye la entrada Recetas
- AND al pulsarla se navega a `/recipes`

#### Scenario: Navegación a Recetas

- GIVEN la barra superior
- WHEN el usuario pulsa Recetas
- THEN navega a `/recipes`

#### Scenario: Estado activo

- GIVEN la ruta `/recipes` activa
- WHEN se renderiza la barra superior
- THEN la entrada Recetas tiene `aria-current="page"`
- AND el resto de entradas no lo tiene

### Requirement: Selector de tema

La aplicación SHALL ofrecer tres modos de tema: Oscuro, Claro y Sistema. El modo por defecto SHALL ser Oscuro. La preferencia SHALL persistirse en `localStorage` y SHALL aplicarse antes del primer pintado para evitar un destello de tema incorrecto. En modo Sistema SHALL seguir `prefers-color-scheme` y reaccionar a sus cambios.

#### Scenario: Tema por defecto

- GIVEN un usuario sin preferencia guardada
- WHEN abre la aplicación
- THEN el tema es oscuro

#### Scenario: Persistencia

- GIVEN el usuario elige Claro
- WHEN recarga la aplicación
- THEN el tema sigue siendo claro

#### Scenario: Modo Sistema

- GIVEN el modo Sistema con el sistema en oscuro
- WHEN el sistema cambia a claro
- THEN la interfaz cambia a claro sin recargar

#### Scenario: Sin destello

- GIVEN la aplicación cargando
- WHEN se pinta el primer frame
- THEN no se muestra el tema claro antes del oscuro

### Requirement: Página de ajustes

La aplicación SHALL exponer la ruta `/settings` accesible desde la entrada Configuración de la barra superior. La página SHALL contener el selector de tema y la identidad del usuario con la acción de cierre de sesión.

#### Scenario: Acceso a ajustes

- GIVEN la barra superior
- WHEN el usuario pulsa Configuración
- THEN navega a `/settings`

#### Scenario: Cambio de tema en ajustes

- GIVEN la página de ajustes
- WHEN el usuario cambia el tema
- THEN la preferencia se guarda y se aplica de inmediato

#### Scenario: Salir desde ajustes

- GIVEN la página de ajustes
- WHEN el usuario pulsa Salir
- THEN se invoca el cierre de sesión
