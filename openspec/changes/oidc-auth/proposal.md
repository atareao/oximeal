# Change: Fase 4 — Autenticación OIDC con Pocket ID

## Intent
Cerrar la instancia self-hosted de OxiMeal, que hoy está **completamente abierta** (CORS `Any`, sin autenticación — issue #5), mediante OIDC contra Pocket ID v2.19.1. El backend, que ya sirve el SPA con `rust-embed` desde el mismo origen, actúa como **cliente confidencial** y BFF: hace el *authorization code flow* con PKCE y `state`/`nonce`, intercambia el código **en servidor**, guarda los tokens en una sesión server-side y expone al navegador **solo una cookie `httpOnly`**. Así el navegador nunca ve tokens. Tras este change, todas las rutas mutantes y de lectura de datos exigen sesión o token válido.

## Scope
- **BFF/sesión**: `GET /auth/login`, `GET /auth/callback`, `GET /auth/me`, `POST /auth/logout`; sesión server-side con cookie `httpOnly` + `Secure` + `SameSite=Lax`, rotación de identificador y renovación transparente con refresh token.
- **Autorización por permisos** del modelo *APIs and permissions* de Pocket ID: validación de access tokens JWT (RFC 9068) contra JWKS, `iss`, `aud` (el resource del API), `exp`/`nbf` y algoritmo; tabla de permisos por endpoint (`read:recipes`, `write:recipes`, `read:plan`, `write:plan`, `read:shopping`, `write:shopping`).
- **Autorización en doble vía**: cookie de sesión (SPA) o `Authorization: Bearer` (Valet u otros clientes), con la misma tabla de permisos.
- **Roles**: los grupos de Pocket ID mapean a roles (`admin`, `member`); la puerta de entrada se controla con *Allowed user groups* del cliente en Pocket ID.
- **Agent API M2M**: los cinco endpoints `/api/agent/*` se consumen con `client_credentials` de un cliente confidencial, con permisos dedicados (`agent:plan`, `agent:shopping`, `agent:read`).
- **CSRF**: cabecera `X-CSRF-Token` obligatoria en mutaciones autenticadas por cookie.
- **Frontend**: guard de rutas, consulta de `/auth/me`, manejo de 401/403 y UI consciente del rol.
- **Despliegue local**: `compose` con Pocket ID + su base de datos y checklist de configuración.

## Out of scope
- **PWA/offline** (service worker, IndexedDB, cola de escrituras): pasa a **Fase 5**.
- **Cambios en el esquema**: NO se añade `user_id`; un único dataset compartido por instancia (uso familiar).
- **API key estática** (`OXIMEAL_API_KEY`, issue #5): queda obsoleta en favor de OIDC; su retirada se documenta.
- Proveedores de identidad distintos de Pocket ID.
- Orquestador/LLM/chat dentro de OxiMeal (sigue siendo Valet, Fase 3).

## Impact
- **Detalle funcional nuevo**: las capacidades `auth-session`, `api-authorization`, `agent-api-auth`, `auth-ui` y `oidc-config`.
- **Cambio de comportamiento global**: CORS deja de ser `Any` y pasa a lista configurable; todas las rutas salvo `GET /api/health`, el OpenAPI y la Swagger UI pasan a exigir credenciales.
- **Datos**: nueva tabla de sesiones server-side en el SQLite del proyecto (sin cambios en las tablas de dominio).
- **Backend**: nuevo middleware de autenticación/autorización, cliente OIDC confidencial (discovery + JWKS con caché), gestión de sesión y CSRF.
- **Frontend**: contexto de sesión, guard de rutas, redirección a login y cabecera CSRF.
- **Configuración/despliegue**: nuevas variables de entorno (`OIDC_*`, `ALLOWED_ORIGINS`, `SESSION_*`), `compose` de desarrollo y reverse proxy HTTPS para redirect URIs fuera de localhost.
- **Dependencia futura**: los access tokens caducan a los 60 min y el refresh a los 30 días, por lo que la **Fase 5** deberá tolerar sesión caducada sirviendo datos cacheados.
- **Trazabilidad**: cierra el issue #5 (autenticación/API key) y desbloquea la exposición segura de la Agent API a Valet.
