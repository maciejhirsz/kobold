use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use wasm_bindgen_test::*;

use super::*;

use crate::js::browser_js::run_npm_lib;

async fn onclick_pjs_process_async() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    run_npm_lib().await
}

#[wasm_bindgen_test]
async fn test_onclick_pjs_process() {
    let expected = "0x00";

    let actual = run_npm_lib().await.unwrap().as_string().unwrap();

    assert_eq!(actual, expected);
}
