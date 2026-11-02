# pwa-shell Specification

## Purpose
Define el comportamiento instalable y offline-first del shell de OxiMeal: un manifiesto web y unos iconos que permiten instalar la aplicación, y un Service Worker de origen propio que precachea el shell y sirve los activos estáticos, dejando las peticiones de la API fuera de su alcance.

## Requirements

### Requirement: Manifiesto instalable
La aplicación SHALL exponer un manifiesto web declarado desde `index.html` con `name`, `short_name`, `start_url`, `scope`, `display: standalone`, `theme_color`, `background_color`, `lang: es` e iconos de 192×192, 512×512 y uno *maskable* de 512×512.

#### Scenario: Manifiesto servido con MIME correcto
- GIVEN un binario con la feature `embed-ui` y `frontend/dist` presente
- WHEN se solicita `GET /manifest.webmanifest`
- THEN se devuelve `200` con `Content-Type: application/manifest+json`

#### Scenario: Instalación de la aplicación
- GIVEN un navegador compatible con la instalación de PWA
- WHEN se carga la aplicación con conexión
- THEN el navegador reconoce el manifiesto y ofrece instalarla
- AND los iconos incluyen una variante *maskable* para Android

### Requirement: Iconos de la aplicación
El proyecto SHALL incluir `icons/icon-192.png`, `icons/icon-512.png`, `icons/icon-512-maskable.png` y `icons/apple-touch-icon-180.png`, referenciados desde el manifiesto y desde `index.html`.

#### Scenario: Icono en iOS
- GIVEN la aplicación añadida a la pantalla de inicio en iOS
- WHEN el sistema muestra su icono
- THEN se usa `icons/apple-touch-icon-180.png`

### Requirement: Service Worker de activos
El proyecto SHALL registrar un Service Worker de origen propio (`/sw.js`, scope `/`) que precachee el shell, sirva los activos con hash con estrategia *cache-first* y las navegaciones con *network-first* y fallback al `index.html` cacheado. El Service Worker SHALL NOT interceptar peticiones bajo `/api/`.

#### Scenario: Recarga sin conexión
- GIVEN una visita previa con conexión en la que el Service Worker se instaló
- WHEN el usuario recarga la aplicación sin conexión
- THEN el shell se sirve desde la caché y la aplicación arranca

#### Scenario: Activo con hash desde caché
- GIVEN un activo bajo `assets/` ya cacheado
- WHEN se solicita de nuevo
- THEN se sirve desde la caché sin acudir a la red

#### Scenario: Petición de API no interceptada
- GIVEN una petición a `/api/shopping-list`
- WHEN el Service Worker la intercepta
- THEN la reenvía a la red sin cachearla

#### Scenario: Registro solo en producción
- GIVEN una ejecución de `pnpm dev`
- WHEN arranca la aplicación
- THEN el Service Worker no se registra

### Requirement: Actualización del Service Worker
El backend SHALL servir `sw.js` con `Cache-Control: no-cache` y el Service Worker SHALL versionar su caché, de modo que una versión nueva sustituya a la anterior y limpie las cachés obsoletas.

#### Scenario: Revalidación del Service Worker
- GIVEN un `sw.js` servido por el backend
- WHEN se inspeccionan sus cabeceras
- THEN incluye `Cache-Control: no-cache`

#### Scenario: Limpieza de cachés antiguas
- GIVEN un Service Worker activándose con una `CACHE_VERSION` nueva
- WHEN se ejecuta el evento `activate`
- THEN las cachés con la versión anterior se eliminan
