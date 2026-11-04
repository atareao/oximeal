#!/usr/bin/env bash
# Tests de los scripts de CI en .github/scripts (sin dependencias externas).
#
# Uso: bash .github/scripts/test-gh-scripts.sh
# Sale con código != 0 si algún caso falla. Se ejecuta en CI (job «Scripts»).
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0
ok() { printf '  ok    %s\n' "$1"; pass=$((pass + 1)); }
ko() {
  printf '  FAIL  %s\n' "$1"
  fail=$((fail + 1))
}

# fixture <workspace> <backend> <package.json> <primera.version.CHANGELOG>
# Crea $tmp/fx simulando la disposición del repo. La sección del CHANGELOG
# incluye una subsección vacía (### Cambiado) que debe omitirse en las notas.
fixture() {
  rm -rf "$tmp/fx"
  mkdir -p "$tmp/fx/frontend" "$tmp/fx/backend"
  printf '[workspace.package]\nversion = "%s"\n' "$1" > "$tmp/fx/Cargo.toml"
  printf '[package]\nname = "oximeal-backend"\nversion = "%s"\n' "$2" > "$tmp/fx/backend/Cargo.toml"
  printf '{\n  "name": "x",\n  "version": "%s"\n}\n' "$3" > "$tmp/fx/frontend/package.json"
  cat > "$tmp/fx/CHANGELOG.md" <<EOF
# Changelog

## [$4] - 2026-01-01

### Añadido

- Algo nuevo.

### Cambiado

### Corregido

- Un arreglo.

## [0.0.1] - 2020-01-01

- Viejo.
EOF
}

run_check() { (cd "$tmp/fx" && bash "$here/check-version.sh" "$1"); }
run_notes() { (cd "$tmp/fx" && bash "$here/release-notes.sh" "$1" "$2"); }

# check <desc> <ok|fail> <patrón-esperado> <comando...>
check() {
  local desc="$1" want="$2" pattern="$3"
  shift 3
  local out rc
  out="$("$@" 2>&1)"
  rc=$?
  local verdict=1
  if [ "$want" = ok ]; then [ "$rc" -eq 0 ] || verdict=0; else [ "$rc" -ne 0 ] || verdict=0; fi
  if [ -n "$pattern" ] && ! grep -qF -- "$pattern" <<<"$out"; then verdict=0; fi
  if [ "$verdict" = 1 ]; then
    ok "$desc"
  else
    ko "$desc"
    printf '        rc=%s\n%s\n' "$rc" "$out" | sed 's/^/        /'
  fi
}

expect_line() { # <desc> <fichero> <regex>
  if grep -qE -- "$3" "$2"; then ok "$1"; else ko "$1"; fi
}
reject_line() { # <desc> <fichero> <regex>
  if grep -qE -- "$3" "$2"; then ko "$1"; else ok "$1"; fi
}

echo "== check-version.sh =="
fixture 1.2.3 1.2.3 1.2.3 1.2.3
check "tag coherente con las cuatro fuentes" ok "tag=1.2.3 cargo=1.2.3 backend=1.2.3" run_check v1.2.3
check "tag sin prefijo v falla (SemVer)" fail "SemVer" run_check 1.2.3
check "tag y workspace incoherentes" fail "!= tag" run_check v9.9.9

fixture 1.2.3 9.9.9 1.2.3 1.2.3
check "backend/Cargo.toml incoherente se detecta" fail "backend/Cargo.toml (9.9.9) != tag (1.2.3)" run_check v1.2.3

fixture 1.2.3 1.2.3 9.9.9 1.2.3
check "frontend/package.json incoherente se detecta" fail "frontend/package.json (9.9.9)" run_check v1.2.3

fixture 1.2.3 1.2.3 1.2.3 9.9.9
check "CHANGELOG.md incoherente se detecta" fail "CHANGELOG.md (9.9.9)" run_check v1.2.3

echo "== release-notes.sh =="
fixture 1.2.3 1.2.3 1.2.3 1.2.3
check "notas generadas desde el CHANGELOG" ok "Notas de la release generadas" run_notes v1.2.3 "$tmp/fx/out.md"
expect_line "incluye el encabezado de versión" "$tmp/fx/out.md" '^## \[1\.2\.3\] - 2026-01-01$'
expect_line "incluye las entradas" "$tmp/fx/out.md" '^- Algo nuevo\.$'
expect_line "conserva la subsección con contenido" "$tmp/fx/out.md" '^### Corregido$'
reject_line "omite la subsección vacía (### Cambiado)" "$tmp/fx/out.md" '^### Cambiado$'
reject_line "no arrastra la versión anterior" "$tmp/fx/out.md" '0\.0\.1|Viejo'

check "sin sección en el CHANGELOG no falla" ok "No hay sección" run_notes v9.9.9 "$tmp/fx/out2.md"
expect_line "escribe una nota mínima con el tag" "$tmp/fx/out2.md" '^Release 9\.9\.9$' 

echo
printf 'Total: %d ok, %d fallos\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
