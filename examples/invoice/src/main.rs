// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;
use kobold_macros::derive_struct_var_index_fn;
use kobold::branching::{Branch2, Branch3};
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::KoboldQR;
use bevy_reflect::{FromReflect, Reflect, DynamicStruct};
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

// macro
derive_struct_var_index_fn!();

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

// use bevy_reflect https://crates.io/crates/bevy_reflect
#[derive(Reflect, FromReflect, Debug)]
pub struct Details {
    inv_date: String,
    from_org_addr: String
}

#[component]
fn EntryView<'a>(state: &'a Hook<State>) -> impl View + 'a {
    // find a specific value
    debug!("rows {:#?}", state.details.table.rows());
    debug!("columns {:#?}", state.details.table.columns());

    let mut label;
    let mut val;
    let valid_labels = ["inv_date", "inv_no", "from_attn_name", "from_org_name", "from_org_addr", "from_email", "to_attn_name", "to_title", "to_org_name", "to_email"];
    let mut details = Details {
        inv_date: String::from("01.01.1970"),
        from_org_addr: String::from("unknown"),
    };
    let mut dynamic_struct = DynamicStruct::default();
    // `state.details.table` only has labels in `columns[col]` and data in its `rows[0][col]`
    for col in state.details.table.columns() {
        let row = 0;
        debug!("col {:#?}", col);
        label = state.details.table.source.get_text(&state.details.table.columns[col]);
        val = state.details.table.source.get_text(&state.details.table.rows[row][col]);
        debug!("col {:#?} - label / val - {:#?} / {:#?}", col, label, val);
        // TODO - replace with derive macro since below isn't valid rust
        if valid_labels.contains(&label) {
            // use https://crates.io/crates/bevy_reflect to emulate `details[`${label}`] = val`
            // that is possible in JavaScript since Rust dot notation is not adequate
            // *details.get_field_mut::<String>(label).unwrap() = val;
            dynamic_struct.insert(label, val.to_string());
        }

        // call macro
        // debug!("struct_var_index_fn {:#?}", struct_var_index_fn());
    }
    details.apply(&dynamic_struct);
    debug!("details {:#?}", details);

    // we know `state.details.table.rows[0][4]` corresponds to `from_org_addr`
    let value = state.details.table.source.get_text(&state.details.table.rows[0][4]);
    debug!("description{:#?}", value);

    if state.entry.editing == true {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.details.table.rows[0][4] = Text::Owned(e.target().value().into());
            state.entry.editing = false;
        });

        let onblur = state.bind(move |state, e: Event<InputElement>| {
            if e.target().value() != "" {
                state.update(e.target().value())
            }
        });

        let onmouseover = state.bind(move |state, e: MouseEvent<InputElement>| {
            let _ = e.target().focus();
        });

        let onkeypress = state.bind(move |state, e: KeyboardEvent<InputElement>| {
            if e.key() == "Enter" && e.target().value() != "" {
                state.update(e.target().value());

                Then::Render
            } else {
                Then::Stop
            }
        });

        Branch2::A(view! {
            <div.edit>
                { ref value }
                <input.edit
                    value={ ref value }
                    type="text"
                    placeholder="<Enter biller address>"
                    {onchange}
                    {onmouseover}
                    {onkeypress}
                    {onblur}
                />
            </div>
        })
    } else {
        let ondblclick = state.bind(move |s, _| s.entry.editing = true);
        let editing = class!("editing" if state.entry.editing);

        Branch2::B(view! {
            <div .todo.{editing}>
                <div .view>
                    <label {ondblclick} >
                        { ref value }
                    </label>
                </div>
            </div>
        })
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
