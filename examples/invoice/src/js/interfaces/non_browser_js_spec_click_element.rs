use wasm_bindgen::prelude::*;

// interface
//
// Note: currently only supports loading .js files located in the root of the Rust project directory.
// See https://github.com/rustwasm/wasm-bindgen/tree/main/examples/import_js/crate
#[wasm_bindgen(module = "/__kobold_click_element.js")]
extern "C" {
    // function
    pub fn __kobold_click_element() -> bool;
}
