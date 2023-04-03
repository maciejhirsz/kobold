// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;
use kobold::branching::Branch3;
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::KoboldQR;
use gloo_console::{console_dbg};
use gloo_utils::format::JsValueSerdeExt;
use log::{info, debug, error, warn};
use serde::{Serialize, Deserialize};
use web_sys::HtmlInputElement as InputElement;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::throw_str;

mod csv;
mod state;

use state::{Editing, State, Table, Text};

#[component]
fn Editor() -> impl View {
    stateful(State::mock, |state| {
        debug!("Editor()");

        let onload_details = {
            let signal = state.signal();

            move |e: Event<InputElement>| {
                let file = match e.target().files().and_then(|list| list.get(0)) {
                    Some(file) => file,
                    None => return,
                };

                signal.update(|state| state.details.name = file.name());

                let signal = signal.clone();

                spawn_local(async move {
                    if let Ok(table) = csv::read_file(file).await {
                        debug!("details.table{:#?}", table);
                        signal.update(move |state| state.details.table = table);
                    }
                })
            }
        };

        let onload_main = {
            let signal = state.signal();

            move |e: Event<InputElement>| {
                let file = match e.target().files().and_then(|list| list.get(0)) {
                    Some(file) => file,
                    None => return,
                };

                signal.update(|state| state.main.name = file.name());

                let signal = signal.clone();

                spawn_local(async move {
                    if let Ok(table) = csv::read_file(file).await {
                        // TODO - find a better way to get the QR code from the file
                        let serialized = serde_json::to_string(&table).unwrap();
                        let value = JsValue::from_serde(&serialized).unwrap();
                        debug!("table {:#?}", value);
                        let payload: Table = serde_json::from_str(&serialized).unwrap();
                        debug!("payload {:#?}", &payload.source.source);

                        let data: &str = &payload.source.source.to_string();
                        // find qr column index
                        let index = &data.find("qr").unwrap();
                        debug!("index {:#?}", *&index);
                        let slice = &data[..*index as usize];
                        debug!("slice {:#?}", slice);
                        let qr_column_count = slice.to_string().matches(",").count();
                        debug!("column of qr {:#?}", qr_column_count+1);

                        // get first row of data below header
                        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=6195d6ef278d9552eba9f8d8a7d457d6
                        let start_bytes: usize = data.find("\n").unwrap();
                        let end_bytes: usize = data[(start_bytes+1)..].find("\n").unwrap();
                        debug!("start_bytes {:#?}", start_bytes);
                        debug!("end_bytes {:#?}", end_bytes);
                        let index_end_next_row = start_bytes + 1 + end_bytes; // where +1 is to skip the `\n`
                        debug!("index_end_next_row {:#?}", index_end_next_row);
                        let row = &data[(start_bytes+1)..][..end_bytes];
                        debug!("row {:#?}", row);
                        let (qr_row_idx, qr_row_str) = row.match_indices(",").nth(qr_column_count-1).unwrap();
                        debug!("qr_row_idx {:#?}", qr_row_idx);
                        let slice_qr = &row[(qr_row_idx+1)..];
                        debug!("slice_qr {:#?}", slice_qr);
                        let qr_code = slice_qr.to_string();

                        debug!("main.table{:#?}", table);
                        signal.update(move |state| state.main.table = table);
                        signal.update(move |state| state.qr_code = qr_code);
                    }
                })
            }
        };

        bind! { state:
            let onkeydown = move |event: KeyboardEvent<_>| {
                if matches!(event.key().as_str(), "Esc" | "Escape") {
                    state.editing = Editing::None;

                    Then::Render
                } else {
                    Then::Stop
                }
            };
        }

        view! {
            <div .invoice-wrapper>
                <section .invoiceapp>
                    <header .header>
                        <h1>"Invoice"</h1>
                    </header>
                    <section .main>
                        <div #input-file-select>
                            <h1>{ ref state.details.name }</h1>
                            <input type="file" accept="text/csv" onchange={onload_details} />
                        </div>
                        <EntryView {state} />
                        <div #input-file-select>
                            <h1>{ ref state.main.name }</h1>
                            <input type="file" accept="text/csv" onchange={onload_main} />
                        </div>
                        <table {onkeydown}>
                            <thead>
                                <tr>
                                {
                                    for state.main.table.columns().map(|col| view! { <Head {col} {state} /> })
                                }
                                </tr>
                            </thead>
                            <tbody>
                            {
                                for state.main.table.rows().map(move |row| view! {
                                    <tr>
                                    {
                                        for state.main.table.columns().map(move |col| view! {
                                            <Cell {col} {row} {state} />
                                        })
                                    }
                                    </tr>
                                })
                            }
                            </tbody>
                        </table>
                    </section>
                    <section .qr>
                        <QRExample />
                    </section>
                </section>
                <footer .info>
                    <p>"Hint: Double-click to edit an invoice field"</p>
                </footer>
            </div>
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.main.table.source.get_text(&state.main.table.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.columns[col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head {onchange} value={ ref value } />
            </th>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Column { col });

        view! { <th {ondblclick}>{ ref value }</th> }
    }
}

#[component]
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.main.table.source.get_text(&state.main.table.rows[row][col]);

    if state.editing == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.rows[row][col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        Branch3::A(view! {
            <td.edit>
                { ref value }
                <input.edit {onchange} value={ ref value } />
            </td>
        })
    // https://github.com/maciejhirsz/kobold/issues/51
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

        if value.contains("0x") {
            Branch3::B(view! {
                <td {ondblclick}>
                    { ref value }
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

#[component]
fn EntryView<'a>(state: &'a Hook<State>) -> impl View + 'a {

    debug!("rows {:#?}", state.details.table.rows());
    debug!("columns {:#?}", state.details.table.columns());
    // let val = state.details.table.source.get_text(&state.details.table.rows[1][0]);
    // let val = state.details.table.source.get_text(&state.columns[0][0]);
    // debug!("val {:#?}", val);
    for row in state.details.table.rows() {
        debug!("row {:#?}", row);
        for col in state.details.table.columns() {
            debug!("col {:#?}", col);
            // 
            // debug!("row {:#?} col {:#?}, val {:#?}", row, col, val.to_string());
            // debug!("row {:#?} col {:#?}", row, col);
        }
    }

    let entry = &state.entry;
    let input = state.entry_editing.then(move || {
        bind! { state:
            let onkeypress = move |event: KeyboardEvent<InputElement>| {
                if event.key() == "Enter" && event.target().value() != "" {
                    state.update(event.target().value());

                    Then::Render
                } else {
                    Then::Stop
                }
            };

            let onblur = move |event: Event<InputElement>| {
                if event.target().value() != "" {
                    state.update(event.target().value())
                }
            };
        }

        let onmouseover = move |event: MouseEvent<InputElement>| {
            let _ = event.target().focus();
        };

        view! {
            <input .edit
                type="text"
                placeholder="<Enter biller address>"
                value={ref entry.description}
                {onmouseover}
                {onkeypress}
                {onblur}
            />
        }
    });

    bind! {
        state:
        let edit = move |_| state.edit_entry();
    }
    let editing = class!("editing" if entry.entry_editing);

    view! {
        <div .todo.{editing}>
            <div .view>
                <label ondblclick={edit} >
                    { ref entry.description }
                </label>
            </div>
            { input }
        </div>
    }
}

#[component]
fn QRExample() -> impl View {
    stateful("Enter something", |data| {
        bind! {
            data:

            let onkeyup = move |event: KeyboardEvent<HtmlTextAreaElement>| *data = event.target().value();
        }

        view! {
            <h1>"QR code example"</h1>
            <KoboldQR {data} />
            <textarea {onkeyup}>{ static data.as_str() }</textarea>
        }
    })
}

#[component]
fn QRForTask(value: &str) -> impl View + '_ {
    let data = &value;

    view! {
        <KoboldQR {data} />
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    debug!("main()");
    kobold::start(view! {
        <Editor />
    });
}
