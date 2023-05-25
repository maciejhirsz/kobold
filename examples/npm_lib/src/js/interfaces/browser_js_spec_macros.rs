use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        ($crate::js::interfaces::browser_js_spec_macros::log(&format_args!($($t)*).to_string()))
    }
}
