use wasm_bindgen::prelude::*;

// interfaces
use crate::js::interfaces::browser_js_spec_alert;
use crate::js::interfaces::non_browser_js_spec_class_example as class_example;
use crate::js::interfaces::non_browser_js_spec_save_file as save_file;
use crate::js::interfaces::non_browser_js_spec_input_empty_value as input_empty_value;
#[macro_use]
use crate::js::interfaces::browser_js_spec_macros;

// https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    check_window();
    greet("hi".to_string().as_str());

    run_non_browser_js();

    Ok(())
}

#[wasm_bindgen]
pub fn greet(name: &str) -> JsValue {
    let age: JsValue = 4.into();
    // browser_js_spec_alert::alert(&format!("Hello, {:?}! {:?}", name, &age));
    return age;
}

#[wasm_bindgen]
pub fn check_window() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
}

#[wasm_bindgen]
pub fn run_non_browser_js() {
    // https://rustwasm.github.io/docs/wasm-bindgen/examples/console-log.html
    crate::console_log!("class_example::name {:?}", class_example::name());
    let x = class_example::MyClass::new();
    assert_eq!(x.number(), 42);
    x.set_number(10);
    crate::console_log!("class_example::MyClass::render() {:?}", &x.render());
}

#[wasm_bindgen]
pub fn run_save_file(filename: &str, data: &[u8]) {
    let has_saved_file = save_file::kobold_save_file(filename, data);
    crate::console_log!(
        "save_file::kobold_save_file(filename, data) {:?}",
        has_saved_file
    );
}

#[wasm_bindgen]
pub fn run_input_empty_value(el_id: &str) {
    input_empty_value::kobold_input_empty_value(el_id);
}
