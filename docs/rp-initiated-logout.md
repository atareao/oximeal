# rp-initiated-logout

## Qué es
El cierre de sesión de OxiMeal deja de dejar viva la sesión SSO del IdP. Al pulsar «Salir», el backend destruye la sesión/cookie y devuelve un `logout_url` de *RP-initiated logout* (Pocket ID); el SPA limpia su estado y su caché y navega a esa URL.

## Ficheros implicados
- `backend/src/auth/routes.rs` — handler `logout` y struct `LogoutResponse`.
- `backend/src/auth/oidc.rs` — helper `logout_url` y validación de `endpoint_url`.
- `backend/src/auth/mod.rs` — timeouts del cliente `reqwest` (`connect_timeout`/`timeout`).
- `backend/src/routes/openapi.rs` — esquema `LogoutResponse` del contrato OpenAPI.
- `frontend/src/api/client.ts` — tipo `LogoutResponse` y `logoutAuth`.
- `frontend/src/auth/navigate.ts` — navegación a una URL externa.
- `frontend/src/auth/SessionProvider.tsx` — limpieza de estado/caché y navegación.

## Cómo funciona
- Contrato: `POST /auth/logout` responde `200` con `{"logout_url": string|null}`.
- Orden exacto en el handler: leer la sesión → copiar `id_token` → `flush()` (destruir sesión y cookie) → construir la URL con `tokio::time::timeout` de 3 s sobre `discovery()`.
- Devuelve `null` cuando: no hay sesión, el IdP no publica `end_session_endpoint`, la discovery falla o supera el timeout, o OIDC está desactivado (la ruta no se monta → `404`).
- La URL incluye `id_token_hint` solo si la sesión guardó un `id_token`; siempre incluye `client_id` y `post_logout_redirect_uri`.
- Frontend: limpia CSRF + caché de React Query + estado; la persistida se limpia de forma *best-effort*; navega con `window.location.assign` cuando `logout_url` no es `null`.
- CSRF sigue exigiéndose en `POST /auth/logout`.

## Variables / configuración
Reutiliza `OIDC_POST_LOGOUT_REDIRECT_URI` (sin variables nuevas).

Recordatorio operativo: la *Logout callback URL* del cliente en Pocket ID debe incluir exactamente esa misma URI.

## Cómo habilitar / desplegar
Reconstruir la imagen y recrear el contenedor. En co1: Dockge → *Update* y verificar que tras «Salir» se vuelve al login sin reentrar solo.

## Decisiones
- Devolver la URL en el JSON en vez de redirigir con `303` desde el backend: el SPA necesita limpiar su estado antes de navegar.
- Destruir la sesión (y la cookie) **antes** del I/O de red, para no depender de la respuesta del IdP.
- Rechazar esquemas distintos de `http`/`https` al construir las URLs.
- Fallback `null` cuando no se puede cerrar el SSO: mantiene el comportamiento anterior (logout local).

## Follow-ups
- Test del timeout de 3 s con un mock de discovery retardado.
- Cifrado en reposo de `sessions.data` (backlog).
- TTL de la caché de discovery.
