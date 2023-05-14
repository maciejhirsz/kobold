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
    let expected = JsValue::from_str("0x00");

    let promise: Promise = future_to_promise(async {
        let val = onclick_pjs_process_async().await.unwrap();
        Ok(val)
    });

    // Convert that promise into a future and make the test wait on it.
    let actual = JsFuture::from(promise).await.unwrap();

    assert_eq!(actual, expected);
}
