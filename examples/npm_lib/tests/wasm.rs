use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use wasm_bindgen_test::*;

use super::*;

use crate::js::browser_js::run_npm_lib;

#[wasm_bindgen_test]
async fn test_onclick_pjs_process() {
    let expected = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3";

    let actual = run_npm_lib().await.unwrap().as_string().unwrap();

    assert_eq!(actual, expected);
}
