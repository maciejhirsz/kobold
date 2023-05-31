// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement};

use kobold::prelude::*;

mod js;

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
