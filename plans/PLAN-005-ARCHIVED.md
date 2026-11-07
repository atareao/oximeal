# PLAN-005 — RP-initiated logout

## Objetivo
Que «Salir» cierre también la sesión SSO de Pocket ID: el backend devuelve un `logout_url` de *RP-initiated logout* y el SPA limpia estado/caché y navega a ella.

## Estado
Completado y archivado (OpenSpec `rp-initiated-logout` fusionado en `openspec/specs/`). Release 0.8.1 publicada (tag `v0.8.1`, imagen GHCR publicada) y change archivado. **Despliegue en co1 completado** con Dockge Update y verificado de extremo a extremo (el cierre de sesión incluye el IdP).

## Alcance
- Backend: handler `logout` (`LogoutResponse`), helper `logout_url`/`endpoint_url`, timeouts del cliente `reqwest`, contrato en el OpenAPI.
- Frontend: tipo `LogoutResponse`, `logoutAuth`, navegación externa, `SessionProvider`.
- Sin variables de entorno nuevas (reutiliza `OIDC_POST_LOGOUT_REDIRECT_URI`).

## Tareas (TDD + SDD)
- [x] Spec OpenSpec `rp-initiated-logout` creada y validada.
- [x] Backend TDD RED→GREEN→REFACTOR (tests de logout, helper de URL, timeouts, rechazo de esquema).
- [x] Frontend TDD (contrato, navegación, fallbacks).
- [x] Auditorías `@rust-reviewer` y `@react-reviewer` con hallazgos corregidos.
- [x] `validate --all --strict`.
- [x] Release 0.8.1 (tag `v0.8.1`, imagen GHCR publicada).
- [x] Despliegue en co1 con Dockge Update — **completado** (verificado el 2026-11-07).
- [x] Archivar el change (`openspec archive`).
