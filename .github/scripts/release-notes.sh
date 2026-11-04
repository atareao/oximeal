#!/usr/bin/env bash
# Genera las notas de la GitHub Release a partir de la sección curada del
# CHANGELOG.md (fuente de verdad, ADR-33).
#
# ¿Por qué no git-cliff? `main` integra cada release por *squash-merge*, así
# que el rango `v<anterior>..v<actual>` en `main` contiene un único commit y
# `git-cliff --current` no puede reconstruir las notas. `git-cliff` se usa solo
# como *fallback* si el CHANGELOG no tuviera la sección de la versión. La salida
# reproduce la sección tal cual, incluido su encabezado `## [X.Y.Z] - fecha`.
set -euo pipefail

version="${1:-}"
out="${2:-RELEASE_NOTES.md}"

if [[ -z "$version" ]]; then
  echo "::error::Falta la versión (p. ej. v0.5.0)."
  exit 1
fi

# Normaliza 'vX.Y.Z' o 'refs/tags/vX.Y.Z' -> 'X.Y.Z'
version="${version##*/}"
version="${version#v}"

body=""
if grep -qE "^## \[${version//./\\.}\]" CHANGELOG.md; then
  body="$(awk -v ver="$version" '
    $0 ~ "^## \\[" ver "\\]" { print; print ""; inside=1; next }
    inside && /^## \[/        { exit }
    inside                    { buf[++n]=$0 }
    END {
      for (i=1; i<=n; i++) {
        line=buf[i]
        # Omite encabezados de subsección sin contenido (p. ej. "### Cambiado").
        if (line ~ /^### /) {
          j=i+1
          while (j<=n && buf[j] ~ /^[[:space:]]*$/) j++
          if (j>n || buf[j] ~ /^### /) continue
        }
        print line
      }
    }
  ' CHANGELOG.md | cat -s | sed -e '/./,$!d')"
fi

if [[ -z "${body//[[:space:]]/}" ]]; then
  echo "::warning::No hay sección '## [$version]' con contenido en CHANGELOG.md; se intenta git-cliff."
  if ! git-cliff --current --strip header -o "$out"; then
    echo "::warning::git-cliff tampoco pudo generar las notas; se usa una nota mínima."
    printf 'Release %s\n' "$version" > "$out"
  fi
  exit 0
fi

printf '%s\n' "$body" > "$out"
echo "✅ Notas de la release generadas desde CHANGELOG.md ($(wc -l < "$out") líneas)"
