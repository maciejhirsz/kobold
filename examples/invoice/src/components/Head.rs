// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::HtmlInputElement as InputElement;

use kobold::prelude::*;

use crate::state::{Editing, State, Text};

#[component(auto_branch)]
pub fn Head(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    debug!("Head get_text source {:?} {:?}", col, row);
    let value: &str;
    if col <= (state.details.table.columns.len() - 1) {
        value = state
            .main
            .table
            .source
            .get_text(&state.main.table.columns[col]);
    } else {
        value = &"";
    }

    if state.editing_main == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.columns[col] = Text::Owned(e.target().value().into());
            state.store();
            state.editing_main = Editing::None;
        });

        view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head
                    {onchange}
                    value={ ref value }
                />
            </th>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing_main = Editing::Column { col });

        view! { <th {ondblclick}>{ ref value }</th> }
    }
}
