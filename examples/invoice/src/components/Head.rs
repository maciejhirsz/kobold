// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::{HtmlElement, HtmlInputElement as InputElement};

use kobold::branching::Branch3;
use kobold::prelude::*;

use crate::state::{Editing, State, Text};

#[component]
pub fn Head(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    // debug!("Head get_text source {:?} {:?}", col, row);
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

    let onadd_row = state.bind(move |state, event: MouseEvent<HtmlElement>| {
        let row = match event.target().get_attribute("data") {
            Some(r) => r,
            None => return,
        };
        let row_usize = match row.parse::<usize>() {
            Ok(r) => r,
            Err(e) => return,
        };

        state.add_row_main(row_usize);
    });

    if state.editing_main == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.columns[col] = Text::Owned(e.target().value().into());
            state.store();
            state.editing_main = Editing::None;
        });

        Branch3::A(view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head
                    {onchange}
                    value={ ref value }
                />
            </th>
        })
    } else {
        let ondblclick = state.bind(move |s, _| s.editing_main = Editing::Column { col });

        if col == (state.main.table.columns.len() - 1) {
            Branch3::B(view! {
                <th {ondblclick}>{ ref value }</th>
                // for the add row button column
                <th.add-container>
                    <button.add
                        data={row}
                        onclick={onadd_row}
                    />
                </th>
                // for the destroy row button column
                <th.destroy-container></th>
            })
        } else {
            Branch3::C(view! {
                <th {ondblclick}>{ ref value }</th>
            })
        }
    }
}
