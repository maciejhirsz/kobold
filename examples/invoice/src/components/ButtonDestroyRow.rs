// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;

use web_sys::HtmlElement;

use crate::state::State;

#[component]
pub fn ButtonDestroyRow(row: usize, state: &Hook<State>) -> impl View {
    view! {
        <button.destroy
            data={row}
            onclick={
                state.bind(move |state, event: MouseEvent<HtmlElement>| {
                    let row = match event.target().get_attribute("data") {
                        Some(r) => r,
                        None => return,
                    };
                    let row_usize = match row.parse::<usize>() {
                        Ok(r) => r,
                        Err(e) => return,
                    };

                    state.destroy_row_main(row_usize);
                })
            }
        />
    }
}
