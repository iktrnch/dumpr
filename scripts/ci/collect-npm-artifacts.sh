#!/usr/bin/env bash
set -euo pipefail

version="$(scripts/ci/release-metadata.sh)"
mapfile -t expected_artifacts < <(scripts/ci/npm-artifact-names.sh "$version")
mkdir npm-dist
find npm-packages -type f -name '*.tgz' -exec cp -- {} npm-dist/ \;
test "$(find npm-dist -maxdepth 1 -type f -name '*.tgz' | wc -l)" -eq "${#expected_artifacts[@]}"

for artifact in "${expected_artifacts[@]}"; do
    test -f "npm-dist/$artifact"
done
