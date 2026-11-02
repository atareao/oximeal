# ui-shell Specification

## Purpose
Define el shell responsive mobile-first de la PWA de OxiMeal: en desktop muestra un panel lateral (`Layout.Sider`) y en móvil una navegación inferior con las pestañas Menú, Recetas, Compra y Asistente, con objetivos táctiles de al menos 48px, tema de Ant Design vía `ConfigProvider` y respeto de los safe-area insets. La pestaña Asistente abre o embebe Valet mediante `VITE_VALET_URL`, sin chat nativo.

## Requirements

### Requirement: Shell responsive
La aplicación SHALL adaptar su navegación al tamaño de pantalla: en desktop SHALL mostrar un `<Layout.Sider>` con panel lateral, y en móvil SHALL mostrar una navegación inferior fija con tres pestañas —Menú, Compra y Asistente—.

#### Scenario: Navegación en desktop
- GIVEN un viewport de ancho igual o superior al breakpoint desktop
- WHEN se carga la aplicación
- THEN se muestra el panel lateral (`Layout.Sider`) con las secciones de navegación
- AND no se muestra la navegación inferior fija

#### Scenario: Navegación en móvil
- GIVEN un viewport de ancho inferior al breakpoint desktop
- WHEN se carga la aplicación
- THEN se muestra la navegación inferior fija con las pestañas Menú, Compra y Asistente
- AND el panel lateral permanece oculto

### Requirement: Objetivos táctiles
En móvil, las filas y controles interactivos SHALL tener una altura mínima de 48px para garantizar una interacción táctil cómoda.

#### Scenario: Control interactivo en móvil
- GIVEN la vista de lista de la compra en un viewport móvil
- WHEN se inspecciona una fila o control interactivo
- THEN su altura es de al menos 48px

### Requirement: Tema Antd y safe-area
La aplicación SHALL configurar Ant Design mediante `ConfigProvider` con tokens de tema, y SHALL respetar los safe-area insets del dispositivo en móvil.

#### Scenario: Aplicación del tema
- GIVEN la aplicación cargada
- WHEN se renderiza el árbol de componentes
- THEN los componentes Antd heredan los tokens definidos en `ConfigProvider`
- AND el layout móvil aplica los safe-area insets en los bordes superior e inferior

### Requirement: Pestaña Asistente
La pestaña Asistente SHALL abrir o embeber Valet usando una URL configurable mediante la variable de entorno `VITE_VALET_URL`, sin implementar chat nativo.

#### Scenario: Apertura de Valet
- GIVEN `VITE_VALET_URL` configurada
- WHEN el usuario pulsa la pestaña Asistente
- THEN se abre o embebe Valet en la URL configurada
- AND OxiMeal no muestra un chat propio

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
