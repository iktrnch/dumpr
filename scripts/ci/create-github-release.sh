#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <tag> <asset-directory>" >&2
    exit 2
}

tag="$1"
asset_directory="$2"
repository="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

mapfile -t assets < <(find "$asset_directory" -type f | sort)
test "${#assets[@]}" -gt 0
gh release create "$tag" \
    --repo "$repository" \
    --verify-tag \
    --generate-notes \
    "${assets[@]}"
