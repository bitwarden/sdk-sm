#!/usr/bin/env bash

# Move to the root of the repository
cd "$(dirname "$0")"
cd ../../

if [ "$1" != "-r" ]; then
  echo "Building in debug mode"
  RELEASE_FLAG=""
  BUILD_DIR="debug"
else
  echo "Building in release mode"
  RELEASE_FLAG="--release"
  BUILD_DIR="release"
fi

# Build
cargo build -p bitwarden-wasm --target wasm32-unknown-unknown ${RELEASE_FLAG}
wasm-bindgen --target bundler --out-dir languages/js/wasm ./target/wasm32-unknown-unknown/${BUILD_DIR}/bitwarden_wasm.wasm
wasm-bindgen --target nodejs --out-dir languages/js/wasm/node ./target/wasm32-unknown-unknown/${BUILD_DIR}/bitwarden_wasm.wasm

# Optimize size
wasm-opt -Os ./languages/js/wasm/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/bitwarden_wasm_bg.wasm
wasm-opt -Os ./languages/js/wasm/node/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/node/bitwarden_wasm_bg.wasm
