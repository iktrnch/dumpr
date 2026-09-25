#!/usr/bin/env bash
# Extract and package the exact tested Linux binary with nFPM.
set -euo pipefail

[[ $# -eq 3 ]] || {
    echo "usage: $0 <input-directory> <amd64|arm64> <output-directory>" >&2
    exit 2
}

input_directory="$1"
tar -xzf "$input_directory"/dumpr-*-x86_64-unknown-linux-musl.tar.gz -C "$input_directory"

package_binary() {
    [[ $# -eq 3 ]] || {
        echo "usage: $0 <input-directory> <amd64|arm64> <output-directory>" >&2
        exit 2
    }

    local binary="$1"
    local arch="$2"
    local output_directory="$3"
    local version
    local config

    version="$(scripts/ci/release-metadata.sh)"
    mkdir -p "$output_directory"
    config="$(mktemp)"
    trap 'rm -f "$config"' EXIT
    sed \
        -e "s|@VERSION@|$version|g" \
        -e "s|@ARCH@|$arch|g" \
        -e "s|@BINARY@|$binary|g" \
        packaging/nfpm.yaml.in > "$config"
    nfpm package --config "$config" --packager deb --target "$output_directory/dumpr_${version}_${arch}.deb"
    nfpm package --config "$config" --packager rpm --target "$output_directory/dumpr-${version}-1.${arch}.rpm"
    rm -f "$config"
    trap - EXIT
}

package_binary "$input_directory"/dumpr-*-x86_64-unknown-linux-musl/dumpr "$2" "$3"
