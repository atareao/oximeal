# Tasks — Configuración y Salir siempre visibles (TDD: RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Confirmar el desbordamiento: calcular el ancho mínimo del header y comprobar que supera 360px con los valores actuales de `AppHeader.css` (verificación: anotar cálculo en el PR; sin cambios de código)
- [x] 1.2 Localizar los breakpoints y clases afectadas en `AppHeader.css` y el uso de `useMediaQuery` en `AppHeader.tsx` (verificación: lectura acotada)

## 2. RED

- [x] 2.1 Añadir test en `AppHeader.test.tsx`: con `useMediaQuery` devolviendo `false` para `(min-width: 480px)` el logo no muestra el texto «OxiMeal» y conserva el nombre accesible (verificación: `npx vitest run frontend/src/components/AppHeader.test.tsx` falla)
- [x] 2.2 Añadir test: Configuración y Salir están en el mismo grupo de acciones y Salir es el último control (verificación: test en rojo si no existe el agrupamiento esperado)
- [x] 2.3 Añadir test: con `status` distinto de `authenticated` no se renderiza el botón «Salir» (verificación: test en rojo si hoy se renderiza en `disabled`)
- [x] 2.4 Ejecutar la suite y confirmar RED (verificación: `npx vitest run` con los nuevos tests fallando y el resto verde)

## 3. GREEN

- [x] 3.1 En `AppHeader.tsx`, introducir la consulta `(min-width: 480px)` y renderizar el logo compacto (`aria-label="OxiMeal"`, texto oculto) cuando no se cumpla (verificación: test 2.1 en verde)
- [x] 3.2 En `AppHeader.css`, añadir `flex-shrink: 0` a `.app-header__actions`, y breakpoint `max-width: 479px` con gaps y padding reducidos y estilo del logo compacto (verificación: revisión de CSS; test 2.2 en verde)
- [x] 3.3 Ajustar el render de `Salir` para depender explícitamente de `status === 'authenticated'` (ya es así) y cubrirlo con el test 2.3 (verificación: test 2.3 en verde)
- [x] 3.4 `npx vitest run` y `npx tsc --noEmit` (verificación: todo verde, sin errores de tipos)

## 4. REFACTOR

- [x] 4.1 Revisar nombres de clases y evitar duplicar la lógica de media queries (verificación: `rg -n '480|compact' frontend/src/components/AppHeader.*`)
- [x] 4.2 `npm run lint` (verificación: sin warnings nuevos)
- [x] 4.3 Re-ejecutar `npx vitest run` (verificación: sin regresiones)
- [x] 4.4 `npx --yes @fission-ai/openspec@1.16.0 validate --all --strict` (verificación: 0 errores)
