#!/usr/bin/env bash
set -euo pipefail

runner_temp="${RUNNER_TEMP:?RUNNER_TEMP is required}"
version="$(scripts/ci/release-metadata.sh)"
source_url="https://github.com/iktrnch/dumpr/archive/refs/tags/v$version.tar.gz"
source_sha="$(curl -fsSL "$source_url" | sha256sum | cut -d' ' -f1)"
binary_url="https://github.com/iktrnch/dumpr/releases/download/v$version/dumpr-$version-x86_64-unknown-linux-musl.tar.gz"
binary_sha="$(sha256sum native/dumpr-*-x86_64-unknown-linux-musl.tar.gz | cut -d' ' -f1)"

for kind in source bin; do
    repo=dumpr
    [[ "$kind" == bin ]] && repo=dumpr-bin
    git clone "ssh://aur@aur.archlinux.org/$repo.git" "$repo"
    if [[ "$kind" == source ]]; then
        url="$source_url"
        sha="$source_sha"
    else
        url="$binary_url"
        sha="$binary_sha"
    fi
    stage="$(mktemp -d "$runner_temp/aur-$kind.XXXXXX")"
    chmod 0777 "$stage"
    trap 'rm -rf "$stage"' EXIT
    docker run --rm \
        -v "$PWD:/work:ro" \
        -v "$stage:/output" \
        -w /work \
        archlinux:base-devel bash -ec "
        pacman -Syu --noconfirm --needed base-devel cargo python
        useradd -m builder
        su builder -c 'cd /work && scripts/ci/generate-aur.sh $kind /output $url $sha x86_64'
      "
    install -m 0644 "$stage/PKGBUILD" "$repo/PKGBUILD"
    install -m 0644 "$stage/.SRCINFO" "$repo/.SRCINFO"
    rm -rf "$stage"
    trap - EXIT
    git -C "$repo" add PKGBUILD .SRCINFO
    git -C "$repo" \
        -c user.name='dumpr release bot' \
        -c user.email='release@users.noreply.github.com' \
        commit -m "update to $version"
    git -C "$repo" push
done
