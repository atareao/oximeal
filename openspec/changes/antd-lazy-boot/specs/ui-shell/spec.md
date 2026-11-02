# Capability: ui-shell

## ADDED Requirements

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
