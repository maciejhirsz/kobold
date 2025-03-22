// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wasm_bindgen::prelude::*;

// Note: only supports loading .js or .mjs files located in the Rust project root.
// See https://github.com/rustwasm/wasm-bindgen/tree/main/examples/import_js/crate
#[wasm_bindgen(module = "/output/koboldNpmLib.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = "koboldNpmLib")]
    pub async fn kobold_npm_lib() -> Result<JsValue, JsValue>;
}
