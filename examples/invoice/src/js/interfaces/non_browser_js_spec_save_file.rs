// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wasm_bindgen::prelude::*;

// interface
//
// Note: currently only supports loading .js files located in the root of the Rust project directory.
// See https://github.com/rustwasm/wasm-bindgen/tree/main/examples/import_js/crate
#[wasm_bindgen(module = "/koboldSaveFile.js")]
extern "C" {
    // function
    #[wasm_bindgen(js_name = "koboldSaveFile")]
    pub fn kobold_save_file(filename: &str, data: &[u8]) -> bool;
}
