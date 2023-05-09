// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement};

use kobold::prelude::*;

mod components;
mod csv;
mod helpers;
mod js;
mod state;
#[cfg(test)]
mod tests;
use components::{
    Cell::Cell, CellDetails::CellDetails, Editor::Editor, Head::Head, HeadDetails::HeadDetails,
};
use state::{Editing, State};

fn main() {
    // Demonstrate use of Rust `wasm-bindgen` https://rustwasm.github.io/docs/wasm-bindgen
    js::browser_js::run();

    wasm_logger::init(wasm_logger::Config::default());
    kobold::start(view! {
        <Editor />
    });
}
