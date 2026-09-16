#!/usr/bin/env bash
# Generate an AUR PKGBUILD and .SRCINFO from Cargo metadata and supplied assets.
# Usage: generate-aur.sh <source|bin> <out-dir> <source-url> <sha256> [arch]
set -euo pipefail
kind="${1:?package kind required}"; out="${2:?output directory required}"
url="${3:?source URL required}"; sha256="${4:?sha256 required}"; arch="${5:-x86_64}"
version="$(scripts/release-metadata.sh)"; template="packaging/aur/PKGBUILD.${kind}.in"
[[ -f "$template" ]] || { echo "unknown AUR package kind: $kind" >&2; exit 2; }
mkdir -p "$out"
case "$arch" in x86_64) target=x86_64-unknown-linux-musl ;; aarch64) target=aarch64-unknown-linux-musl ;; *) echo "unsupported AUR arch: $arch" >&2; exit 2 ;; esac
sed -e "s|@VERSION@|$version|g" -e "s|@URL@|$url|g" -e "s|@SHA256@|$sha256|g" -e "s|@ARCH@|$arch|g" -e "s|@TARGET@|$target|g" "$template" > "$out/PKGBUILD"
(cd "$out" && makepkg --printsrcinfo > .SRCINFO)
