// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wasm_bindgen::prelude::*;
use gloo_console::log;

use crate::js::interfaces::browser_js_spec_npm_lib as connect;

#[wasm_bindgen]
pub async fn run_npm_lib() -> Result<JsValue, JsValue> {
    let hash = connect::kobold_npm_lib().await;
    log!(&format_args!("connect::kobold_npm_lib() {:?}", hash.as_ref()?).to_string());
    hash
}
