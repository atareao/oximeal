# Plan 007 — Pestañas en la vista de detalle de receta

- **Issue:** #125
- **Change OpenSpec:** `add-recipe-view-tabs`
- **Capacidad afectada:** `recipe-catalog` (Requirement «Detalle de receta»)
- **Rama:** `feature/recipe-view-tabs` (desde `development`)
- **Fecha:** 2026-11-09

## 1. Objetivo

Reorganizar la vista de detalle de receta (`/recipes/:id`) en **dos pestañas**:

1. **Instrucciones** — pestaña principal, activa por defecto.
2. **Ingredientes** — segunda pestaña, con el `IngredientEditor` actual.

El resto del detalle (cabecera, meta, selector de raciones/escalado, alertas, botonera de acciones, etiquetas y fuente) permanece **por encima** de las pestañas, sin cambios funcionales.

## 2. Mapa arquitectónico

| Elemento | Fichero | Rol |
|---|---|---|
| Página de detalle | `frontend/src/pages/RecipeDetailPage.tsx` | `RecipeDetailPage` (ruta) + `RecipeDetailContent` (render, líneas ~61-386) |
| Instrucciones (hoy) | Ídem, líneas ~357-364 | `<section aria-labelledby="recipe-instructions-title">` con `<Paragraph>` |
| Ingredientes (hoy) | `frontend/src/components/IngredientEditor.tsx` | Modo lectura (~127-156) y edición en línea (~213-293) |
| Estilos | `frontend/src/pages/RecipeDetailPage.css` | Clases `recipe-detail__*` (~107-214) |
| Test de la página | `frontend/src/pages/RecipeDetailPage.test.tsx` | Vitest + @testing-library/react + user-event (~711 líneas) |
| Router | `frontend/src/App.tsx` | Ruta `recipes/:id` (línea 40) |

**Estado actual:** ingredientes primero, instrucciones después, en un scroll único.

## 3. Decisiones de diseño

- **D1 — Componente de pestañas:** usar `Tabs` de **antd** (ya dependencia, `antd ^6.6.8`). Aporta roles `tablist`/`tab`/`tabpanel` y navegación por teclado sin código propio. Alternativa descartada: componente de tabs a mano (más código y a11y que mantener).
- **D2 — Pestaña por defecto:** `defaultActiveKey="instructions"`.
- **D3 — Paneles montados:** no destruir el panel inactivo (`destroyOnHidden=false`) para conservar el estado de edición de ingredientes al cambiar de pestaña.
- **D4 — Instrucciones vacías:** si `instructions` es `null` o vacío, el panel muestra un estado vacío («Esta receta no tiene instrucciones.»).
- **D5 — Alcance:** sin cambios de backend ni de API; sin reflejar la pestaña en la URL (se deja como follow-up).

## 4. Tareas (TDD)

### 4.1 Frontend — RED

- [x] T1. Añadir en `RecipeDetailPage.test.tsx` el test «Instrucciones activa por defecto»: existe `role=tablist` con pestañas «Instrucciones» e «Ingredientes»; el panel de Instrucciones es visible y el de Ingredientes no.
- [x] T2. Test «cambiar a Ingredientes»: activar la pestaña Ingredientes → se ve la lista de ingredientes (nombres + cantidades, escaladas si procede).
- [x] T3. Test «volver a Instrucciones».
- [x] T4. Test «receta sin instrucciones» → estado vacío en el panel de Instrucciones.
- [x] T5. Test de accesibilidad: presencia y coherencia de roles `tablist`/`tab`/`tabpanel` y `aria-selected`.
- [x] Verificar en RED: `npx vitest run src/pages/RecipeDetailPage.test.tsx` (los nuevos fallan; el resto en verde).

### 4.2 Frontend — GREEN

- [x] T6. En `RecipeDetailPage.tsx`, mover la sección de instrucciones y el `<IngredientEditor>` a dos paneles de un `Tabs` de antd con `defaultActiveKey="instructions"`, situado **por debajo** de etiquetas/acciones y **por encima** de la fuente original.
- [x] T7. Estado vacío de instrucciones (D4).
- [x] T8. Estilos de `Tabs` en `RecipeDetailPage.css` (espaciado y objetivos táctiles ≥48px), manteniendo los nombres `recipe-detail__*`.
- [x] Verificar: `npx vitest run src/pages/RecipeDetailPage.test.tsx` en verde y `npx tsc --noEmit` sin errores.

### 4.3 Frontend — REFACTOR

- [x] T9. Ejecutar `npm run lint`, `npx tsc --noEmit` y `npm run format:check`; corregir avisos.
- [x] T10. Ejecutar `npx vitest run` completo; confirmar 100% en verde (429 existentes + nuevos).

### 4.4 Auditoría

- [ ] T11. Lanzar `@react-reviewer` sobre `RecipeDetailPage.tsx`, `IngredientEditor.tsx`, `RecipeDetailPage.css` y los tests. Corregir hallazgos y re-ejecutar los tests.

### 4.5 Documentación y cierre

- [ ] T12. Crear `docs/recipe-view-tabs.md` (qué es, ficheros implicados, cómo funciona, decisiones y follow-ups).
- [ ] T13. Verificación E2E manual en navegador: abrir una receta, comprobar la pestaña por defecto, cambiar a Ingredientes y volver, editar ingredientes dentro de su pestaña, y comprobar que raciones/escalado siguen intactos.
- [ ] T14. Archivar el change con `openspec archive add-recipe-view-tabs` y renombrar este plan a `plans/PLAN-007-ARCHIVED.md`.

## 5. Criterios de aceptación

- [ ] Al abrir `/recipes/:id` se muestra la pestaña **Instrucciones** activa.
- [ ] Se puede cambiar a **Ingredientes** y viceversa.
- [ ] La edición en línea de ingredientes sigue funcionando dentro de su pestaña.
- [ ] Accesible por teclado y con roles ARIA correctos.
- [ ] `vitest`, `tsc --noEmit` y `eslint` en verde.

## 6. Riesgos

- Los tests existentes dependen del orden del DOM y del texto de instrucciones/ingredientes: adaptar aserciones sin perder cobertura (T4/T6).
- El test importa el CSS con `?raw` y comprueba clases: conservar los nombres `recipe-detail__*`.