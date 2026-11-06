# Tasks — la imagen nunca publica el stub (RED → GREEN → REFACTOR)

## 1. Preparación

- [x] 1.1 Reproducir el fallo con la imagen publicada: `podman pull ghcr.io/atareao/oximeal:0.6.0`, comprobar que el binario mide ~285 KB y que el contenedor sale con código 0 sin logs (verificación: `podman run --rm --entrypoint /bin/sh ... -c 'ls -l /usr/local/bin'` y `podman logs`)
- [x] 1.2 Confirmar el tamaño del binario correcto de la release v0.6.0 (16,8 MB) como referencia (verificación: `tar -tzvf` del `.tar.gz`)

## 2. RED

- [x] 2.1 Añadir al `Dockerfile`, tras el build real y antes del stage `runtime`, una verificación que falle si el binario no supera un mínimo (p. ej. `test "$(stat -c%s target/release/oximeal-backend)" -gt 5000000`) (verificación: construir la imagen con el Dockerfile actual y sin invalidar el stub debe FALLAR)
- [x] 2.2 Documentar en el PR la reproducción del fallo y la causa (mtimes + fingerprint de Cargo) (verificación: sección «Reproducción» en el PR)

## 3. GREEN

- [x] 3.1 Añadir `cargo clean -p oximeal-backend` y `touch backend/src/*.rs backend/build.rs` antes del build definitivo; mantener la caché de dependencias intacta (verificación: `podman build -t oximeal:0.6.1-fix .` termina con código 0)
- [x] 3.2 Verificar el tamaño del binario dentro de la imagen: > 5 MB (verificación: `podman run --rm --entrypoint /bin/sh oximeal:0.6.1-fix -c 'stat -c%s /usr/local/bin/oximeal-backend'`)
- [x] 3.3 Arrancar el contenedor con `OIDC_ENABLED=false`, volumen y puerto alternativo; comprobar `/api/health` 200 y logs de arranque (verificación: `curl -fsS http://127.0.0.1:8081/api/health` y `podman logs`)
- [x] 3.4 Comprobar que la UI embebida se sirve (`/` 200 y assets presentes) (verificación: `curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:8081/`)

## 4. REFACTOR

- [ ] 4.1 Revisar que el `Dockerfile` sigue cacheando dependencias entre builds (segunda construcción sin cambios de código reutiliza la capa de dependencias) (verificación: tiempo/layer cache en un segundo `podman build`)
- [x] 4.2 `openspec validate --all --strict` (verificación: 0 errores)
- [x] 4.3 Actualizar `deploy/README.md` solo si el procedimiento cambia (verificación: sin cambios si no aplica)
