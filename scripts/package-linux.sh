#!/usr/bin/env bash
# Package an already-built Linux binary with nFPM. It never invokes Cargo.
set -euo pipefail
[[ $# -eq 3 ]] || { echo "usage: $0 <binary> <amd64|arm64> <output-dir>" >&2; exit 2; }
binary="$1"; arch="$2"; output_dir="$3"; version="$(scripts/release-metadata.sh)"
mkdir -p "$output_dir"
config="$(mktemp)"; trap 'rm -f "$config"' EXIT
sed -e "s|@VERSION@|$version|g" -e "s|@ARCH@|$arch|g" -e "s|@BINARY@|$binary|g" packaging/nfpm.yaml.in > "$config"
nfpm package --config "$config" --packager deb --target "$output_dir/dumpr_${version}_${arch}.deb"
nfpm package --config "$config" --packager rpm --target "$output_dir/dumpr-${version}-1.${arch}.rpm"
