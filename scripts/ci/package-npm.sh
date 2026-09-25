#!/usr/bin/env bash
set -euo pipefail

version="$(scripts/ci/release-metadata.sh)"
mkdir -p npm-packages

for target in \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-musl \
    aarch64-apple-darwin \
    x86_64-pc-windows-msvc
do
    unpacked="native/unpacked/$target"
    mkdir -p "$unpacked"
    if [[ "$target" == x86_64-pc-windows-msvc ]]; then
        unzip -q "native/native-$target/dumpr-$version-$target.zip" -d "$unpacked"
        executable=dumpr.exe
    else
        tar -xzf "native/native-$target/dumpr-$version-$target.tar.gz" -C "$unpacked"
        executable=dumpr
    fi
    scripts/ci/generate-npm-packages.sh platform "$target" \
        "$unpacked/dumpr-$version-$target/$executable" \
        "npm-packages/$target"
done

scripts/ci/generate-npm-packages.sh wrapper \
    '@iktrnch/dumpr' npm-packages/wrapper-scoped
scripts/ci/generate-npm-packages.sh wrapper \
    dumpr-cli npm-packages/wrapper-unscoped

for package in npm-packages/*; do
    (cd "$package" && npm pack)
done
