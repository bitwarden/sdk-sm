#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REQUIRED_WASM_BINDGEN_VERSION="0.2.105"

pushd "${REPO_ROOT}"
trap 'popd' EXIT

ensure_wasm_bindgen() {
	if ! command -v wasm-bindgen >/dev/null 2>&1; then
		cargo install -f wasm-bindgen-cli --version "${REQUIRED_WASM_BINDGEN_VERSION}"
		return
	fi

	current_version="$(wasm-bindgen --version | awk '{print $2}')"
	if [ "${current_version}" != "${REQUIRED_WASM_BINDGEN_VERSION}" ]; then
		cargo install -f wasm-bindgen-cli --version "${REQUIRED_WASM_BINDGEN_VERSION}"
	fi
}

ensure_wasm_bindgen

PROFILE="debug"
if [ "${1:-}" = "-r" ]; then
	PROFILE="release"
	cargo build -p bitwarden-wasm --target wasm32-unknown-unknown --release
else
	cargo build -p bitwarden-wasm --target wasm32-unknown-unknown
fi

WASM_PATH="./target/wasm32-unknown-unknown/${PROFILE}/bitwarden_wasm.wasm"

wasm-bindgen --target bundler --out-dir languages/js/wasm "${WASM_PATH}"
wasm-bindgen --target nodejs --out-dir languages/js/wasm/node "${WASM_PATH}"

wasm-opt -Os ./languages/js/wasm/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/bitwarden_wasm_bg.wasm
wasm-opt -Os ./languages/js/wasm/node/bitwarden_wasm_bg.wasm -o ./languages/js/wasm/node/bitwarden_wasm_bg.wasm
