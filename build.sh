#!/usr/bin/env bash
# Build the static site into ./dist (works for GitHub Pages sub-paths: all URLs are relative).
# Requires: rust with the wasm32-unknown-unknown target, and wasm-bindgen-cli 0.2.128 (matches Cargo.lock).
set -euo pipefail
cd "$(dirname "$0")"

FLAGS=(--release --target wasm32-unknown-unknown)
if [ -n "${BUILD_STD:-}" ]; then   # only for toolchains without a prebuilt wasm32 std (e.g. distro rustc)
  export RUSTC_BOOTSTRAP=1
  FLAGS+=(-Zbuild-std=std,panic_abort)
fi
cargo build "${FLAGS[@]}"

rm -rf dist && mkdir -p dist/pkg
wasm-bindgen --target web --no-typescript --out-dir dist/pkg \
  target/wasm32-unknown-unknown/release/mopimopi-dioxus.wasm
cp -r public/* dist/
cp web/index.html dist/index.html
touch dist/.nojekyll
echo "built ./dist  ($(du -sh dist | cut -f1))"
