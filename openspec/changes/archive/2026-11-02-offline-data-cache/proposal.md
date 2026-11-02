# Change: Caché de datos offline y modo degradado

## Why
Sin conexión o con la sesión OIDC caducada, la aplicación vacía las vistas o redirige al login. El roadmap exige que la Fase 5 tolere la sesión caducada sirviendo datos cacheados mientras no sea posible reautenticar.

## What Changes
- Cachear en IndexedDB las respuestas de las consultas de datos (lista de la compra, plan del menú y recetas) e hidratar la caché de react-query al arrancar.
- Persistir en IndexedDB tras cada carga correcta, con una lista blanca de claves.
- Introducir un modo degradado: con datos en caché y sin red, o con la sesión caducada estando sin conexión, mostrar los datos con un aviso persistente en lugar de redirigir al login.
- Limpiar la caché de datos de IndexedDB al cerrar sesión.

## Impact
- Capabilities: nueva `offline-data`; delta en `auth-ui`.
- Ficheros: `frontend/src/offline/**`, `frontend/src/auth/SessionProvider.tsx`, `frontend/src/auth/AuthGuard.tsx`, `frontend/src/components/OfflineBanner.tsx`, `frontend/src/main.tsx`, `PLAN.md`, `CHANGELOG.md`.
- Sin dependencias nuevas.

Cierra el issue #30.
