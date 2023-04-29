use wasm_bindgen::prelude::*;

// interface
//
// Note: currently only supports loading .js files located in the root of the Rust project directory.
// See https://github.com/rustwasm/wasm-bindgen/tree/main/examples/import_js/crate
#[wasm_bindgen(module = "/__kobold_class_example.js")]
extern "C" {
    // function
    pub fn name() -> String;

    // class
    pub type MyClass;

    #[wasm_bindgen(constructor)]
    pub fn new() -> MyClass;

    #[wasm_bindgen(method, getter)]
    pub fn number(this: &MyClass) -> u32;

    #[wasm_bindgen(method, setter)]
    pub fn set_number(this: &MyClass, number: u32) -> MyClass;

    #[wasm_bindgen(method)]
    pub fn render(this: &MyClass) -> String;
}
