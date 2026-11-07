# PLAN-005 — RP-initiated logout

## Objetivo
Que «Salir» cierre también la sesión SSO de Pocket ID: el backend devuelve un `logout_url` de *RP-initiated logout* y el SPA limpia estado/caché y navega a ella.

## Estado
Especificado y validado (OpenSpec `rp-initiated-logout`), backend y frontend en TDD completos y auditorías pasadas. Pendiente: release, despliegue y archivo del change.

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
- [ ] Release 0.8.1.
- [ ] Despliegue en co1.
- [ ] Archivar el change (`openspec archive`).
