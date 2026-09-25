#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <target> <true|false>" >&2
    exit 2
}

target="$1"
cross="$2"

if [[ "$cross" == true ]]; then
    cargo zigbuild --release --target "$target"
else
    cargo build --release --target "$target"
fi
