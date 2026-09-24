#!/usr/bin/env bash
# Creates npm packages from already-built executables; it never invokes Cargo.
set -euo pipefail
usage() { echo "usage: $0 wrapper <out> | platform <target> <binary> <out>" >&2; exit 2; }
mode="${1:-}"; shift || true; version="$(scripts/release-metadata.sh)"
case "$mode" in
  wrapper)
    [[ $# -eq 1 ]] || usage; out="$1"; mkdir -p "$out/bin"
    cat > "$out/package.json" <<EOF
{"name":"@iktrnch/dumpr","version":"$version","description":"Native launcher for dumpr","license":"MIT","repository":{"type":"git","url":"https://github.com/iktrnch/dumpr"},"bin":{"dumpr":"bin/dumpr.cjs"},"files":["bin"],"optionalDependencies":{"@iktrnch/dumpr-linux-x64":"$version","@iktrnch/dumpr-linux-arm64":"$version","@iktrnch/dumpr-darwin-x64":"$version","@iktrnch/dumpr-darwin-arm64":"$version","@iktrnch/dumpr-win32-x64":"$version"}}
EOF
    cp packaging/npm/dumpr.cjs "$out/bin/dumpr.cjs" ;;
  platform)
    [[ $# -eq 3 ]] || usage; target="$1"; binary="$2"; out="$3"
    case "$target" in
      x86_64-unknown-linux-musl) package="@iktrnch/dumpr-linux-x64"; os=linux; cpu=x64; exe=dumpr ;;
      aarch64-unknown-linux-musl) package="@iktrnch/dumpr-linux-arm64"; os=linux; cpu=arm64; exe=dumpr ;;
      x86_64-apple-darwin) package="@iktrnch/dumpr-darwin-x64"; os=darwin; cpu=x64; exe=dumpr ;;
      aarch64-apple-darwin) package="@iktrnch/dumpr-darwin-arm64"; os=darwin; cpu=arm64; exe=dumpr ;;
      x86_64-pc-windows-msvc) package="@iktrnch/dumpr-win32-x64"; os=win32; cpu=x64; exe=dumpr.exe ;;
      *) echo "unsupported npm target: $target" >&2; exit 2 ;;
    esac
    mkdir -p "$out/bin"; cp "$binary" "$out/bin/$exe"; chmod +x "$out/bin/$exe" 2>/dev/null || true
    printf '{"name":"%s","version":"%s","description":"Native dumpr binary for %s","license":"MIT","repository":{"type":"git","url":"https://github.com/iktrnch/dumpr"},"os":["%s"],"cpu":["%s"],"files":["bin"]}\n' "$package" "$version" "$target" "$os" "$cpu" > "$out/package.json" ;;
  *) usage ;;
esac
