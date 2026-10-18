#!/usr/bin/env bash
# TensorLab — macOS / Linux 打包脚本
# 用法（在仓库根执行）：
#   ./packaging/build-unix.sh
#   ./packaging/build-unix.sh --bundle
set -euo pipefail

VERSION="0.2.1"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BUNDLE=0
for arg in "$@"; do
  case "$arg" in
    --bundle) BUNDLE=1 ;;
    *) echo "unknown argument: $arg"; exit 2 ;;
  esac
done

echo "== 1/6 fmt =="
cargo fmt --check

echo "== 2/6 layers =="
cargo xtask layers

echo "== 3/6 tests =="
cargo test --workspace -j 2

echo "== 4/6 release build =="
cargo build --release -p tl-cli -p tl-mcp-server -p tl-desktop

echo "== 5/6 assemble portable package =="
if [[ "$(uname -s)" == "Darwin" ]]; then
  PLATFORM="macos-$(uname -m)"
else
  PLATFORM="linux-$(uname -m)"
fi
OUT="dist/tensorlab-${VERSION}-${PLATFORM}"
rm -rf "$OUT"
mkdir -p "$OUT"
cp target/release/tensorlab "$OUT"/
cp target/release/tl-mcp-server "$OUT"/
cp target/release/tensorlab-desktop "$OUT"/
cp -R toolbox demos "$OUT"/
cp README.md CHANGELOG.md ROADMAP.md "$OUT"/
cp docs/attribution.md "$OUT"/ATTRIBUTION.md
tar -czf "${OUT}.tar.gz" -C dist "$(basename "$OUT")"
echo "portable package -> ${OUT}.tar.gz"

if [[ "$BUNDLE" == "1" ]]; then
  echo "== 6/6 tauri installers =="
  if ! command -v cargo-tauri >/dev/null 2>&1; then
    echo "warning: cargo-tauri not installed; run: cargo install tauri-cli --version '^2'" >&2
  else
    ( cd apps/tl-desktop && cargo tauri build )
    echo "installers -> target/release/bundle/"
  fi
else
  echo "== 6/6 skipped installers (pass --bundle) =="
fi
