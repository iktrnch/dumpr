#!/usr/bin/env bash
set -euo pipefail

VERSION="$(cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys
metadata = json.load(sys.stdin)
package = next(package for package in metadata["packages"] if package["name"] == "dumpr")
print(package["version"])
')"

TAG="${1:?usage: scripts/check-release-version.sh vX.Y.Z}"
TAG="${TAG#v}"

[[ "$VERSION" == "$TAG" ]] || {
    echo "Version mismatch:"
    echo "Cargo.toml: $VERSION"
    echo "Tag:        $TAG"
    exit 1
}

echo "Release version: $VERSION"
