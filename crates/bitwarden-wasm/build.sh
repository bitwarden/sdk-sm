#!/usr/bin/env bash
set -eo pipefail

# Move to the root of the repository
REPO_ROOT="$(git rev-parse --show-toplevel)"
pushd "$REPO_ROOT" >/dev/null || exit 1

if [ "$1" != "-r" ]; then
  echo "Building in debug mode"
  RELEASE_FLAG=""
  BUILD_FOLDER="debug"
else
  echo "Building in release mode"
  RELEASE_FLAG="--release"
  BUILD_FOLDER="release"
fi

# Build with MVP CPU target, two reasons:
# 1. It is required for wasm2js support
# 2. While webpack supports it, it has some compatibility issues that lead to strange results
# Note that this requirest build-std which is an unstable feature,
# this normally requires a nightly build, but we can also use the
# RUSTC_BOOTSTRAP hack to use the same stable version as the normal build
RUSTFLAGS=-Ctarget-cpu=mvp RUSTC_BOOTSTRAP=1 cargo build -p bitwarden-wasm -Zbuild-std=panic_abort,std --target wasm32-unknown-unknown ${RELEASE_FLAG}

# Generate the wasm-bindgen bindings
wasm-bindgen --target bundler --out-dir languages/js/wasm ./target/wasm32-unknown-unknown/${BUILD_FOLDER}/bitwarden_wasm.wasm
wasm-bindgen --target nodejs --out-dir languages/js/wasm/node ./target/wasm32-unknown-unknown/${BUILD_FOLDER}/bitwarden_wasm.wasm

# Optimize size
wasm-opt -Os ./languages/js/wasm/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/bitwarden_wasm_bg.wasm
wasm-opt -Os ./languages/js/wasm/node/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/node/bitwarden_wasm_bg.wasm

# Transpile to JS
wasm2js -Os ./languages/js/wasm/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/bitwarden_wasm_bg.wasm.js
npx terser ./languages/js/wasm/bitwarden_wasm_bg.wasm.js -o ./languages/js/wasm/bitwarden_wasm_bg.wasm.js

popd >/dev/null
