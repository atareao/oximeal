# Design

## Context

El flujo de login OIDC descubre el documento del proveedor y guarda
`end_session_endpoint` en `OidcProvider`, pero el logout actual
(`POST /auth/logout`) solo hace `session.flush()`, limpia la cookie y responde
204. El frontend, al pasar a `anonymous`, redirige a `/auth/login`; el IdP
mantiene su sesión SSO y reautentica sin intervención, por lo que el usuario no
percibe un cierre real.

La sesión server-side (`AuthenticatedSession`) ya conserva el `id_token`, y la
configuración valida `OIDC_POST_LOGOUT_REDIRECT_URI`. Ambas piezas permiten
construir una URL de RP-initiated logout sin cambios de configuración.

## Decisions

- **URL en JSON (200) en lugar de un 303 directo desde el backend.** El handler
  devuelve `{"logout_url": string|null}` tras destruir la sesión. Alternativa
  descartada: responder 303 `Location: <end_session_endpoint>` desde el backend,
  que rompe el contrato actual de `fetch`, impide al frontend limpiar su caché
  antes de salir y no distingue el caso «sin OIDC» sin una URL artificial.
- **Destruir la sesión y limpiar la cookie antes de construir la URL.** Garantiza
  que un fallo al construir la URL no deja una sesión viva, y que el
  `id_token_hint` se lee de una sesión que ya no puede reutilizarse. La URL se
  construye a partir de los datos copiados antes de destruir la sesión.
- **Usar `id_token_hint` más `client_id` y `post_logout_redirect_uri`.** Es lo que
  espera la especificación de RP-initiated logout y lo que valida Pocket ID
  contra sus *Logout callback URLs*. Alternativa descartada: enviar solo
  `client_id` y `post_logout_redirect_uri`, que algunos IdP rechazan por falta de
  contexto de sesión.
- **Fallback `null` en lugar de error.** Sin OIDC, sin `end_session_endpoint` o
  sin sesión autenticada, el logout local sigue siendo válido; devolver `null`
  permite al frontend conservar el flujo actual (anónimo → login) sin tratar el
  caso como fallo.
- **Navegación de navegador en el frontend.** `window.location.assign` sale del
  contexto SPA y permite al IdP mostrar su pantalla o redirigir directamente.
  Alternativa descartada: `fetch` al `end_session_endpoint`, bloqueado por CORS y
  sin efecto sobre la sesión SSO del navegador.

## Risks

- **Dependencia del IdP**: si Pocket ID no publica `end_session_endpoint` o
  rechaza la `post_logout_redirect_uri`, el cierre remoto no ocurre; el fallback
  local mantiene la aplicación utilizable.
- **Tokens en tránsito**: el `id_token_hint` viaja en la URL hacia el IdP (uso
  previsto por OIDC), pero no debe registrarse en logs ni incluirse en el cuerpo.
  El cuerpo solo contiene `logout_url`.
- **CSRF y sesión ya destruida**: el guard se evalúa antes del handler; una
  petición sin `X-CSRF-Token` sigue devolviendo 403 y no revela si había sesión.
- **Cambio de contrato**: clientes que asumían 204 en `POST /auth/logout` deben
  aceptar 200 con cuerpo; solo lo consume el frontend de este repositorio.