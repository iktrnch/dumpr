#!/usr/bin/env bash
# Creates npm packages from already-built executables; it never invokes Cargo.
set -euo pipefail

usage() {
  echo "usage: $0 wrapper <package-name> <out> | platform <target> <binary> <out>" >&2
  exit 2
}

render_template() {
  local template="$1" output="$2"
  shift 2

  local -a substitutions=()
  while [[ $# -gt 0 ]]; do
    [[ $# -ge 2 ]] || { echo "missing value for template placeholder: $1" >&2; exit 2; }
    substitutions+=( -e "s|@$1@|$2|g" )
    shift 2
  done

  sed "${substitutions[@]}" "$template" > "$output"
  if grep -Eq '@[A-Z0-9_]+@' "$output"; then
    echo "unresolved placeholder in generated manifest: $output" >&2
    exit 1
  fi
}

mode="${1:-}"
shift || true
version="$(scripts/release-metadata.sh)"

case "$mode" in
  wrapper)
    [[ $# -eq 2 ]] || usage
    package_name="$1"
    out="$2"
    case "$package_name" in
      @iktrnch/dumpr|dumpr-cli) ;;
      *) echo "unsupported npm wrapper package: $package_name" >&2; exit 2 ;;
    esac
    mkdir -p "$out/bin"
    render_template packaging/npm/package.wrapper.json.in "$out/package.json" \
      PACKAGE_NAME "$package_name" \
      VERSION "$version"
    cp packaging/npm/dumpr.cjs "$out/bin/dumpr.cjs"
    ;;
  platform)
    [[ $# -eq 3 ]] || usage
    target="$1"
    binary="$2"
    out="$3"
    case "$target" in
      x86_64-unknown-linux-musl) package_name="@iktrnch/dumpr-linux-x64"; os=linux; cpu=x64; exe=dumpr ;;
      aarch64-unknown-linux-musl) package_name="@iktrnch/dumpr-linux-arm64"; os=linux; cpu=arm64; exe=dumpr ;;
      aarch64-apple-darwin) package_name="@iktrnch/dumpr-darwin-arm64"; os=darwin; cpu=arm64; exe=dumpr ;;
      x86_64-pc-windows-msvc) package_name="@iktrnch/dumpr-win32-x64"; os=win32; cpu=x64; exe=dumpr.exe ;;
      *) echo "unsupported npm target: $target" >&2; exit 2 ;;
    esac
    mkdir -p "$out/bin"
    cp "$binary" "$out/bin/$exe"
    chmod +x "$out/bin/$exe" 2>/dev/null || true
    render_template packaging/npm/package.platform.json.in "$out/package.json" \
      PACKAGE_NAME "$package_name" \
      VERSION "$version" \
      TARGET "$target" \
      OS "$os" \
      CPU "$cpu"
    ;;
  *) usage ;;
esac
