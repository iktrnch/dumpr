#!/usr/bin/env bash
set -euo pipefail
command -v actionlint >/dev/null || { echo "actionlint is required; see README.md" >&2; exit 127; }
actionlint
