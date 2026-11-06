# Capability: deployment

## MODIFIED Requirements

### Requirement: Imagen multi-stage reproducible

El repositorio SHALL incluir un `Dockerfile` en la raíz (contexto de build = raíz
del monorepo) con **tres stages**: `frontend` (Node + pnpm), `builder` (Rust) y
`runtime` (imagen mínima). La imagen SHALL construirse como
`ghcr.io/atareao/oximeal` (etiqueta por versión/commit). El stage `runtime` SHALL
contener el binario compilado con el código real del repositorio y **nunca** el
stub usado para calentar la caché de dependencias. El build SHALL invalidar los
artefactos del stub antes de la compilación definitiva y SHALL fallar si el
binario resultante no supera una verificación mínima de tamaño.

#### Scenario: Build completo de la imagen

- GIVEN un checkout limpio del repositorio con `Cargo.lock` y `frontend/pnpm-lock.yaml`
- WHEN se ejecuta `docker build -t ghcr.io/atareao/oximeal:local .`
- THEN el stage `frontend` instala con `pnpm install --frozen-lockfile` desde `frontend/pnpm-lock.yaml` y construye `frontend/dist` con `pnpm build`
- AND el stage `builder` compila el binario release con la UI embebida mediante `cargo build --release -p oximeal-backend --features embed-ui`, copiando `frontend/dist` desde el stage `frontend`
- AND el stage `runtime` contiene únicamente el binario y sus dependencias de sistema
- AND el build termina con código 0

#### Scenario: Build reproducible con lockfiles

- GIVEN los lockfiles versionados `Cargo.lock` y `frontend/pnpm-lock.yaml`
- WHEN se repite el build sin modificar el código
- THEN la instalación de pnpm usa exactamente las versiones del lockfile
- AND la compilación de Cargo respeta las versiones de `Cargo.lock`
- AND no se descargan dependencias fuera de los lockfiles

#### Scenario: Falta de lockfile

- GIVEN un checkout sin `frontend/pnpm-lock.yaml` o sin `Cargo.lock`
- WHEN se intenta construir la imagen
- THEN `pnpm install --frozen-lockfile` falla y el build se detiene
- AND no se genera una imagen parcial silenciosa

#### Scenario: El binario final no es el stub de caché

- GIVEN el stub compilado para calentar la caché de dependencias
- WHEN el stage `builder` copia el código real y ejecuta la compilación definitiva
- THEN los artefactos y fingerprints del stub se invalidan antes de compilar
- AND el binario copiado al stage `runtime` corresponde al código real, con un tamaño muy superior al del stub
- AND si el binario final no supera la verificación mínima de tamaño, el build falla

#### Scenario: Contenedor servible

- GIVEN la imagen construida
- WHEN se inicia el contenedor y se consulta `/api/health`
- THEN responde `200` con estado `ok`
- AND los logs muestran el arranque del servidor
