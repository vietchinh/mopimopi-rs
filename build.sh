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
# The stylesheets are `asset!`s, which need assets/tailwind.css to exist when cargo compiles them.
# `dx` generates it itself; this script (which doesn't go through `dx`) generates it here. The file is
# minified because, without `dx`, nothing else will minify it.
if [ -f tailwind.css ] && command -v npx >/dev/null 2>&1; then
  npx @tailwindcss/cli -i tailwind.css -o assets/tailwind.css --minify
  # Tailwind reads src/**/*.rs as plain text; fail if a stray word in the code became a utility (see tailwind.css).
  node tools/check-tailwind.mjs
  # ...and that the block undoing Tailwind's Preflight (tailwind.css) still matches the installed Tailwind.
  node tools/cancel-preflight.mjs --check
else
  echo "error: tailwind.css or npx not found; assets/tailwind.css cannot be generated" >&2; exit 1
fi
cargo build "${FLAGS[@]}"

rm -rf dist && mkdir -p dist/pkg
wasm-bindgen --target web --no-typescript --out-dir dist/pkg \
  target/wasm32-unknown-unknown/release/mopimopi-dioxus.wasm
cp -r public/* dist/
mkdir -p dist/assets && cp assets/*.css dist/assets/ && cp -r assets/font dist/assets/
cp web/index.html dist/index.html

touch dist/.nojekyll
echo "built ./dist  ($(du -sh dist | cut -f1))"
