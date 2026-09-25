#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 1 ]] || {
    echo "usage: $0 <deb|rpm>" >&2
    exit 2
}

package_kind="$1"
expected_version="$(scripts/ci/release-metadata.sh)"

case "$package_kind" in
    deb)
        # shellcheck disable=SC2016
        docker run --rm \
            --env DUMPR_EXPECTED_VERSION="$expected_version" \
            -v "$PWD/dist:/packages:ro" \
            ubuntu:24.04 bash -ec '
            apt-get update; apt-get install -y /packages/*.deb
            test -x /usr/bin/dumpr; dumpr --help
            test "$(dumpr --version)" = "dumpr $DUMPR_EXPECTED_VERSION"
          '
        ;;
    rpm)
        # shellcheck disable=SC2016
        docker run --rm \
            --env DUMPR_EXPECTED_VERSION="$expected_version" \
            -v "$PWD/dist:/packages:ro" \
            fedora:latest bash -ec '
            dnf install -y /packages/*.rpm
            test -x /usr/bin/dumpr; dumpr --help
            test "$(dumpr --version)" = "dumpr $DUMPR_EXPECTED_VERSION"
          '
        ;;
    *)
        echo "unsupported Linux package kind: $package_kind" >&2
        exit 2
        ;;
esac
