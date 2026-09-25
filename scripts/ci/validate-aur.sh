#!/usr/bin/env bash
set -euo pipefail

version="$(scripts/ci/release-metadata.sh)"
mkdir -p aur-fixtures
git archive --format=tar.gz --prefix="dumpr-$version/" -o "aur-fixtures/dumpr-$version.tar.gz" HEAD
cp native/dumpr-*-x86_64-unknown-linux-musl.tar.gz aur-fixtures/dumpr-bin.tar.gz
chmod -R a+rwx aur-fixtures

# shellcheck disable=SC2016
docker run --rm -v "$PWD:/work" -w /work archlinux:base-devel bash -ec '
    pacman -Syu --noconfirm --needed cargo python namcap
    useradd -m builder
    mkdir -p /work/aur-source
    chown -R builder:builder /work/aur-fixtures /work/aur-source
    su builder -c "cd /work && scripts/ci/generate-aur.sh source aur-source file:///work/aur-fixtures/dumpr-'"$version"'.tar.gz $(sha256sum aur-fixtures/dumpr-'"$version"'.tar.gz | cut -d\  -f1)"
    su builder -c "cd /work/aur-source && makepkg -f --noconfirm && namcap *.pkg.tar.zst"
    pacman -U --noconfirm /work/aur-source/*.pkg.tar.zst
    dumpr --help; test "$(dumpr --version)" = "dumpr '"$version"'"
  '

# shellcheck disable=SC2016
docker run --rm -v "$PWD:/work" -w /work archlinux:base-devel bash -ec '
    pacman -Syu --noconfirm --needed cargo python namcap
    useradd -m builder
    mkdir -p /work/aur-bin
    chown -R builder:builder /work/aur-fixtures /work/aur-bin
    su builder -c "cd /work && scripts/ci/generate-aur.sh bin aur-bin file:///work/aur-fixtures/dumpr-bin.tar.gz $(sha256sum aur-fixtures/dumpr-bin.tar.gz | cut -d\  -f1) x86_64"
    su builder -c "cd /work/aur-bin && makepkg -f --noconfirm && namcap *.pkg.tar.zst"
    pacman -U --noconfirm /work/aur-bin/*.pkg.tar.zst
    test -x /usr/bin/dumpr; dumpr --help; test "$(dumpr --version)" = "dumpr '"$version"'"
  '
