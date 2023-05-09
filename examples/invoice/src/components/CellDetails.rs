// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::HtmlInputElement as InputElement;

use kobold::branching::Branch3;
use kobold::prelude::*;

use crate::components::QRForTask::QRForTask;
use crate::state::{Editing, State, Text};

#[component]
pub fn CellDetails(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    // debug!("row/col: {:?}/{:?}", row, col);
    // debug!("CellDetails get_text source {:?} {:?}", col, row);
    let value: &str;
    if col <= (state.details.table.columns.len() - 1) {
        value = state
            .details
            .table
            .source
            .get_text(&state.details.table.rows[row][col]);
    } else {
        value = &"";
    }

    if state.editing_details == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.details.table.rows[row][col] = Text::Owned(e.target().value().into());
            state.store();
            state.editing_details = Editing::None;
        });

        let mut selected = false;

        let onmouseenter = move |e: MouseEvent<InputElement>| {
            if !selected {
                let input = e.target();
                input.focus();
                input.select();
                selected = true;
            }
        };

        Branch3::A(view! {
            <td.edit>
                { ref value }
                <input.edit
                    {onchange}
                    {onmouseenter}
                    value={ ref value }
                />
            </td>
        })
    // https://github.com/maciejhirsz/kobold/issues/51
    } else {
        let ondblclick = state.bind(move |s, _| s.editing_details = Editing::Cell { row, col });

        if value.contains("0x") {
            Branch3::B(view! {
                <td {ondblclick}>
                    <QRForTask {value} />
                </td>
            })
        } else {
            Branch3::C(view! {
                <td {ondblclick}>{ ref value }</td>
            })
        }
    }
}
