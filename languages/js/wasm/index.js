wasm = await import('./bitwarden_wasm_bg.wasm');

import { __wbg_set_wasm } from "./bitwarden_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./bitwarden_wasm_bg.js";
