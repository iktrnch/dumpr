#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <release-tag> <artifact-directory>" >&2
    exit 2
}

case "$1" in
    v*) version="${1#v}" ;;
    *) echo "::error::Expected a v-prefixed release tag, got: $1" >&2; exit 1 ;;
esac
artifact_root="$2"

resolve_exact() {
    local filename="$1"
    mapfile -t matches < <(find "$artifact_root" -type f -name "$filename" | sort)
    if [[ "${#matches[@]}" -ne 1 ]]; then
        echo "::error::Expected exactly one npm artifact named $filename; found ${#matches[@]}" >&2
        printf 'Matched artifact: %s\n' "${matches[@]}" >&2
        return 1
    fi
    resolved="${matches[0]}"
}

mapfile -t expected_filenames < <(scripts/ci/npm-artifact-names.sh "$version")
packages=()
for filename in "${expected_filenames[@]}"; do
    resolve_exact "$filename"
    packages+=("$resolved")
done

mapfile -t all_packages < <(find "$artifact_root" -type f -name '*.tgz' | sort)
if [[ "${#all_packages[@]}" -ne "${#expected_filenames[@]}" ]]; then
    echo "::error::Expected exactly six npm artifacts; found ${#all_packages[@]}" >&2
    printf 'npm artifact: %s\n' "${all_packages[@]}" >&2
    exit 1
fi

for package in "${packages[@]}"; do
    npm publish "$package"
done
