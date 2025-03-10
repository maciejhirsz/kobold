// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use web_sys::HtmlElement;

use kobold::prelude::*;

mod js;

// mod tests;

struct State {
    hash: String,
}

impl State {
    fn new() -> Self {
        State {
            hash: "0x0".to_owned(),
        }
    }
}

async fn onclick_pjs_process(state: Signal<State>, event: MouseEvent<HtmlElement>) {
    let res = js::browser_js::run_npm_lib().await;
        
    let hash = match res.ok().and_then(|value| value.as_string()) {
        Some(hash) => hash,
        None => panic!("error fetching from API"),
    };

    state.update(move |state| state.hash = hash);
}

#[component]
fn NpmLib() -> impl View {
    stateful(State::new, |state| {
        let onclick_pjs = state
            .bind_async(|state, event: MouseEvent<HtmlElement>| onclick_pjs_process(state, event));

        // No need to close tags at the end of the macro
        view! {
            <button type="button" onclick={onclick_pjs}>"Connect"</button>
            <div>{ ref state.hash }</div>
        }
    })
}

fn main() {
    kobold::start(view! {
        <NpmLib />
    });
}

use wasm_bindgen_test::*;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

use crate::js::browser_js::run_npm_lib;

#[wasm_bindgen_test]
async fn test_onclick_pjs_process() {

    let expected = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3";

    let actual = run_npm_lib().await.unwrap().as_string().unwrap();

    assert_eq!(actual, expected);
}
