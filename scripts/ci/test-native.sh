#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <target> <executable>" >&2
    exit 2
}

target="$1"
executable="$2"
version="$(scripts/ci/release-metadata.sh)"
binary="target/$target/release/$executable"

"$binary" --help
test "$("$binary" --version)" = "dumpr $version"
