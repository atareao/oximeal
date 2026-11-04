#!/usr/bin/env bash
# Verifica que el tag de release es coherente con las versiones del repositorio.
# Fuente de verdad: [workspace.package] version de Cargo.toml (ADR-33).
set -euo pipefail

tag="${1:-}"
tag="${tag#refs/tags/}"

if [[ -z "$tag" ]]; then
  echo "::error::No se ha recibido ningún tag."
  exit 1
fi

if [[ ! "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "::error::El tag '$tag' no tiene el formato SemVer 'v<major>.<minor>.<patch>'."
  exit 1
fi

version="${tag#v}"

cargo_version="$(awk -F'"' '/^\[/{s=($0=="[workspace.package]")} s && /^version/{print $2; exit}' Cargo.toml)"
backend_version="$(awk -F'"' '/^\[/{s=($0=="[package]")} s && /^version/{print $2; exit}' backend/Cargo.toml)"
pkg_version="$(grep -m1 -E '"version"' frontend/package.json | sed -E 's/.*"version": *"(.*)".*/\1/')"
changelog_version="$(grep -m1 -E '^## \[[0-9]+\.[0-9]+\.[0-9]+' CHANGELOG.md | sed -E 's/^## \[([^]]+)\].*/\1/')"

echo "tag=$version cargo=$cargo_version backend=$backend_version package.json=$pkg_version changelog=$changelog_version"

fail=0
[[ "$cargo_version" == "$version" ]] || { echo "::error::Cargo.toml ($cargo_version) != tag ($version)"; fail=1; }
[[ "$backend_version" == "$version" ]] || { echo "::error::backend/Cargo.toml ($backend_version) != tag ($version)"; fail=1; }
[[ "$pkg_version" == "$version" ]] || { echo "::error::frontend/package.json ($pkg_version) != tag ($version)"; fail=1; }
[[ "$changelog_version" == "$version" ]] || { echo "::error::CHANGELOG.md ($changelog_version) != tag ($version)"; fail=1; }
[[ "$fail" == 0 ]] || exit 1

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "version=$version" >> "$GITHUB_OUTPUT"
fi
echo "✅ Coherencia de versionado correcta para $tag"
