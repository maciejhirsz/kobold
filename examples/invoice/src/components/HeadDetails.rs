// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::{HtmlInputElement as InputElement};

use kobold::prelude::*;

use crate::state::{Editing, State, Text};

#[component(auto_branch)]
pub fn HeadDetails(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    // debug!("row/col: {:?}/{:?}", row, col);
    let value = state
        .details
        .table
        .source
        .get_text(&state.details.table.rows[row][col]);

    if state.editing_details == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.details.table.rows[row][col] = Text::Owned(e.target().value().into());
            state.store();
            state.editing_details = Editing::None;
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
        let ondblclick = state.bind(move |s, _| s.editing_details = Editing::Cell { row, col });

        view! { <th {ondblclick}>{ ref value }</th> }
    }
}
