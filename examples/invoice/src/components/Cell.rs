// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement, UiEvent};

use kobold::branching::Branch7;
use kobold::prelude::*;

use crate::components::{
    ButtonAddRow::ButtonAddRow, ButtonDestroyRow::ButtonDestroyRow, QRForTask::QRForTask,
};
use crate::js;
use crate::state::{Editing, State, Text};

#[component]
pub fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    // debug!("Cell get_text source {:?} {:?}", col, row);
    let value: &str;
    let row_idx_below_current_row_idx = row + 1;
    if col <= (state.details.table.columns.len() - 1) {
        value = state
            .main
            .table
            .source
            .get_text(&state.main.table.rows[row][col]);
    } else {
        value = &"";
    }

    if state.editing_main == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.rows[row][col] = Text::Owned(e.target().value().into());
            state.store();
            state.editing_main = Editing::None;
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

        // only show remove row button after the last column
        if col == (state.main.table.columns.len() - 1) {
            Branch7::A(view! {
                <td.edit>
                    { ref value }
                    <input.edit
                        {onchange}
                        {onmouseenter}
                        value={ ref value }
                    />
                </td>
                <td.add-container>
                    <ButtonAddRow row={row_idx_below_current_row_idx} {state} />
                </td>
                <td.destroy-container>
                    <ButtonDestroyRow {row} {state} />
                </td>
            })
        } else {
            Branch7::B(view! {
                <td.edit>
                    { ref value }
                    <input.edit
                        {onchange}
                        {onmouseenter}
                        value={ ref value }
                    />
                </td>
            })
        }
    // https://github.com/maciejhirsz/kobold/issues/51
    } else {
        let ondblclick = state.bind(move |s, _| s.editing_main = Editing::Cell { row, col });

        // TODO - should show the delete button regardless of whether the last column contains a QR code
        if value.contains("0x") == true && (col == state.main.table.columns.len() - 1) {
            Branch7::C(view! {
                <td {ondblclick}>
                    <QRForTask {value} />
                </td>
                <td.add-container>
                    <ButtonAddRow row={row_idx_below_current_row_idx} {state} />
                </td>
                <td.destroy-container>
                    <ButtonDestroyRow {row} {state} />
                </td>
            })
        } else if value.contains("0x") == true && (col != state.main.table.columns.len() - 1) {
            Branch7::D(view! {
                <td {ondblclick}>
                    <QRForTask {value} />
                </td>
            })
        } else if value.contains("0x") == false && (col == state.main.table.columns.len() - 1) {
            Branch7::E(view! {
                <td {ondblclick}>{ ref value }</td>
                <td.add-container>
                    <ButtonAddRow row={row_idx_below_current_row_idx} {state} />
                </td>
                <td.destroy-container>
                    <ButtonDestroyRow {row} {state} />
                </td>
            })
        } else if value.contains("0x") == false && (col != state.main.table.columns.len() - 1) {
            Branch7::F(view! {
                <td {ondblclick}>{ ref value }</td>
            })
        } else {
            Branch7::G(view! {
                <td>"error"</td>
            })
        }
    }
}
