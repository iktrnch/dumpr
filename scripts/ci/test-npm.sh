#!/usr/bin/env bash
set -euo pipefail

version="$(scripts/ci/release-metadata.sh)"
package_root="$PWD/npm-packages"
smoke_root="$PWD/npm-smoke"
native_package="$package_root/x86_64-unknown-linux-musl/iktrnch-dumpr-linux-x64-$version.tgz"

smoke_wrapper() {
    local wrapper_package="$1"
    local wrapper_name="$2"
    local smoke_dir="$3"

    mkdir -p "$smoke_dir"
    (
        cd "$smoke_dir"
        npm install --offline --omit=optional --ignore-scripts \
            --no-audit --no-fund --no-save --package-lock=false \
            "$native_package" "$wrapper_package"
        test -f node_modules/@iktrnch/dumpr-linux-x64/package.json
        test "$(node -p "require('./node_modules/$wrapper_name/package.json').name")" = "$wrapper_name"
        node_modules/.bin/dumpr --help
        test "$(node_modules/.bin/dumpr --version)" = "dumpr $version"
    )
}

smoke_wrapper \
    "$package_root/wrapper-scoped/iktrnch-dumpr-$version.tgz" \
    '@iktrnch/dumpr' \
    "$smoke_root/scoped"
smoke_wrapper \
    "$package_root/wrapper-unscoped/dumpr-cli-$version.tgz" \
    dumpr-cli \
    "$smoke_root/unscoped"

cmp \
    "$package_root/wrapper-scoped/bin/dumpr.cjs" \
    "$package_root/wrapper-unscoped/bin/dumpr.cjs"
cmp \
    "$smoke_root/scoped/node_modules/@iktrnch/dumpr-linux-x64/bin/dumpr" \
    "$smoke_root/unscoped/node_modules/@iktrnch/dumpr-linux-x64/bin/dumpr"
