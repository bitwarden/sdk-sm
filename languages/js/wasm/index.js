if (typeof WebAssembly !== "object" || typeof WebAssembly.instantiate !== "function") {
	throw new Error("WebAssembly is required to use @bitwarden/sdk-wasm.");
}

const wasm = await import("./bitwarden_wasm_bg.wasm");

import { __wbg_set_wasm } from "./bitwarden_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./bitwarden_wasm_bg.js";
