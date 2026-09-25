#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <target> <executable>" >&2
    exit 2
}

target="$1"
executable="$2"
version="$(scripts/ci/release-metadata.sh)"
stage="dist/dumpr-$version-$target"

mkdir -p "$stage"
cp "target/$target/release/$executable" "$stage/"
cp README.md LICENSE.md "$stage/"

if [[ "$executable" == dumpr.exe ]]; then
    archive="dumpr-$version-$target.zip"
    (cd dist && 7z a "$archive" "dumpr-$version-$target")
else
    archive="dumpr-$version-$target.tar.gz"
    tar -C dist -czf "dist/$archive" "dumpr-$version-$target"
fi

if command -v sha256sum >/dev/null; then
    (cd dist && sha256sum -- "$archive" > "SHA256SUMS-$target.txt")
else
    (cd dist && shasum -a 256 "$archive" > "SHA256SUMS-$target.txt")
fi
