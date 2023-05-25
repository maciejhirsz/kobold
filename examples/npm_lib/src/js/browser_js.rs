// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wasm_bindgen::prelude::*;

use crate::js::interfaces::browser_js_spec_npm_lib as connect;

#[macro_use]
use crate::js::interfaces::browser_js_spec_macros;

#[wasm_bindgen]
pub async fn run_npm_lib() -> Result<JsValue, JsValue> {
    let hash = connect::kobold_npm_lib().await;
    crate::console_log!("connect::kobold_npm_lib() {:?}", hash);
    hash
}
