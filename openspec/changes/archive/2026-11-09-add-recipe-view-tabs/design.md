# Design

## Decisión: Tabs de antd

Se usa `Tabs` de antd (ya dependencia del proyecto) por sus roles ARIA (`tablist`/`tab`/`tabpanel`) y su navegación por teclado. Se configura `defaultActiveKey="instructions"` y se mantienen los paneles montados al cambiar de pestaña para conservar el estado de edición de ingredientes.

## Alternativas

- Componente de tabs propio: descartado por el coste de accesibilidad y mantenimiento.
- Reflejar la pestaña activa en la URL (`?tab=`): fuera de alcance; se deja como follow-up.

## Estado vacío de instrucciones

`instructions` puede ser `null`; en ese caso el panel muestra «Esta receta no tiene instrucciones.».

## Impacto en tests

Los tests de la página asumen el orden del DOM; se adaptan las aserciones manteniendo la cobertura. El test importa el CSS con `?raw`, por lo que se conservan los nombres de clase `recipe-detail__*`.