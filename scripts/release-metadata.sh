#!/usr/bin/env bash
set -euo pipefail
cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys
metadata = json.load(sys.stdin)
package = next(package for package in metadata["packages"] if package["name"] == "dumpr")
print(package["version"])
'
