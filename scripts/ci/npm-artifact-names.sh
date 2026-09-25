#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 1 ]] || {
    echo "usage: $0 <version>" >&2
    exit 2
}

version="$1"
printf '%s\n' \
    "iktrnch-dumpr-linux-x64-$version.tgz" \
    "iktrnch-dumpr-linux-arm64-$version.tgz" \
    "iktrnch-dumpr-darwin-arm64-$version.tgz" \
    "iktrnch-dumpr-win32-x64-$version.tgz" \
    "iktrnch-dumpr-$version.tgz" \
    "dumpr-cli-$version.tgz"
