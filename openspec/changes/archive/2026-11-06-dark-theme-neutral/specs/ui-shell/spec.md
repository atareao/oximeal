# Spec Delta

## MODIFIED Requirements

### Requirement: Tema Antd y safe-area

La aplicación SHALL configurar Ant Design mediante `ConfigProvider` con tokens dependientes del modo activo (claro u oscuro) y SHALL respetar los safe-area insets en la barra superior y en el contenido. En modo oscuro, la paleta SHALL ser **neutra** (negros y grises, sin tinte azulado perceptible) y SHALL garantizar contraste **AA** (ratio ≥ 4.5:1) entre el texto y sus fondos. El color del chrome (`color-scheme`, `<meta name="theme-color">` y `theme_color` del manifest) SHALL mantenerse coherente con el tema resuelto.

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

#### Scenario: Fondos neutros en modo oscuro

- GIVEN el modo oscuro activo
- WHEN se inspeccionan las variables de superficie (`--bg`, `--surface`, `--surface-muted`) y los tokens de fondo de Ant Design
- THEN usan una escala neutra de grises con canales R, G y B equilibrados
- AND ningún valor de fondo tiene dominante azul (canal B sin exceder de forma perceptible a R y G)

#### Scenario: Contraste AA del texto en modo oscuro

- GIVEN el modo oscuro activo
- WHEN se evalúa el contraste de `--text` y `--text-muted` sobre `--bg`, `--surface` y `--surface-muted`
- THEN cada combinación de texto normal alcanza un ratio de contraste mayor o igual a 4.5:1

#### Scenario: El selector de tema sigue funcionando

- GIVEN el modo oscuro activo
- WHEN el usuario cambia el tema a Claro o Sistema y vuelve a Oscuro
- THEN el tema se aplica de inmediato sin recargar
- AND `color-scheme` refleja el modo resuelto

#### Scenario: Coherencia del theme-color

- GIVEN el modo oscuro activo
- WHEN se resuelve el color del chrome
- THEN el `<meta name="theme-color">` coincide con `theme_color` del manifest
- AND dicho valor corresponde a la paleta oscura activa
