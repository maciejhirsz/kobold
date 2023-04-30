// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::branching::Branch3;
use kobold::prelude::*;
use kobold_qr::KoboldQR;
use log::debug;

use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement};

mod csv;
mod js;
mod state;
mod tests;

use state::{Editing, State, Table, Text};

#[component]
fn Editor() -> impl View {
    stateful(State::default, |state| {
        debug!("Editor()");

        let onload_details = state.bind_async(|state, event: Event<InputElement>| async move {
            debug!("onload_details");
            let file = match event.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            state.update(|state| state.details.filename = file.name());

            if let Ok(table) = csv::read_file(file).await {
                debug!("state.details.table {:#?}", table);
                // https://docs.rs/kobold/latest/kobold/stateful/struct.Signal.html#method.update
                state.update(move |state| {
                    state.details.table = table;
                    state.store(); // update local storage
                });
            }
        });

        let onsave_details = state.bind_async(|state, event: MouseEvent<HtmlElement>| async move {
            // closure required just to debug with access to state fields, since otherwise it'd trigger a render
            state.update_silent(|state| {
                debug!("onsave_details: {:?}", &state.details);
            });

            state.update(|state| {
                // update local storage and state so that &state.details isn't
                // `Content { filename: "\0\0\0\0\0\0\0", table: Table { source: TextSource
                //   { source: "\0" }, columns: [Insitu(0..0)], rows: [] } }`
                state.store();
                match csv::generate_csv_data_for_download(&state.details) {
                    Ok(csv_data) => {
                        debug!("csv_data {:?}", csv_data);
                        // cast String into a byte slice
                        let csv_data_byte_slice: &[u8] = csv_data.as_bytes();
                        js::browser_js::run_save_file(&state.details.filename, csv_data_byte_slice);
                    }
                    Err(err) => {
                        panic!(
                            "failed to generate csv data for download {:?}",
                            state.details.filename
                        );
                    }
                };
                debug!(
                    "successfully generate csv data for download {:?}",
                    state.details.filename
                );
            });
        });

        let onload_main = state.bind_async(|state, event: Event<InputElement>| async move {
            let file = match event.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            state.update(|state| state.main.filename = file.name());

            if let Ok(table) = csv::read_file(file).await {
                state.update(move |state| {
                    state.main.table = table;
                    state.store(); // update local storage
                });
            }
        });

        view! {
            <div .invoice-wrapper>
                <section .invoiceapp>
                    <header .header>
                        <h1>"Invoice"</h1>
                    </header>
                    <section .main>
                        <div class="container">
                            <input type="file" id="file-input" accept="text/csv" onchange={onload_details} />
                            <input type="button" onclick="document.getElementById('file-input').click()" value="Upload CSV file" />
                            <label for="file-input" class="label">{ ref state.details.filename }</label>
                            <button #button-file-save type="button" onclick={onsave_details}>"Save to CSV file"</button>
                            <br />
                        </div>
                        <table
                            onkeydown={
                                state.bind(move |state, event: KeyboardEvent<_>| {
                                    if matches!(event.key().as_str(), "Esc" | "Escape") {
                                        state.editing_details = Editing::None;

                                        Then::Render
                                    } else {
                                        Then::Stop
                                    }
                                })
                            }
                        >
                            <thead.details>
                                <tr>
                                {
                                    for state.details.table.columns().map(|col| view! {
                                        <HeadDetails {col} row={1} {state} />
                                    })
                                }
                                </tr>
                            </thead>
                            <tbody.details>
                                <tr>
                                {
                                    for state.details.table.columns().map(|col| view! {
                                        <CellDetails {col} row={0} {state} />
                                    })
                                }
                                </tr>
                            </tbody>
                        </table>
                        <div #input-file-select>
                            <h1>{ ref state.main.filename }</h1>
                            <input type="file" accept="text/csv" onchange={onload_main} />
                        </div>
                        <table
                            onkeydown={
                                state.bind(move |state, event: KeyboardEvent<_>| {
                                    if matches!(event.key().as_str(), "Esc" | "Escape") {
                                        state.editing_main = Editing::None;

                                        Then::Render
                                    } else {
                                        Then::Stop
                                    }
                                })
                            }
                        >
                            <thead>
                                <tr>
                                {
                                    for state.main.table.columns().map(|col| view! {
                                        <Head {col} row={0} {state} />
                                    })
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
                </section>
                <footer .info>
                {
                    (
                        state.editing_main == Editing::None &&
                        state.editing_details == Editing::None
                    ).then(|| view! {
                        <p>"Hint: Double-click to edit an invoice field"</p>
                    })
                }
                </footer>
            </div>
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state
        .main
        .table
        .source
        .get_text(&state.main.table.columns[col]);

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

#[component]
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state
        .main
        .table
        .source
        .get_text(&state.main.table.rows[row][col]);

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
        let ondblclick = state.bind(move |s, _| s.editing_main = Editing::Cell { row, col });

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

#[component(auto_branch)]
fn HeadDetails(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
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

#[component]
fn CellDetails(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
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

// Credit: maciejhirsz
fn sword(input: &str) -> (&str, &str) {
    let (left, right) = match input.split_once('|') {
        Some(res) => res,
        None => panic!("unable to sword"),
    };

    (left.trim(), right.trim())
}

#[component]
fn QRForTask(value: &str) -> impl View + '_ {
    let (left, right): (&str, &str) = sword(value);
    // assert_eq!(&v, &Vec::from(["0x100", "h160"]));
    debug!("{:#?} {:#?}", &left, &right);
    let data: &str = left;
    let format: &str = right;

    view! {
        <div.qr>
            <KoboldQR data={data} />
            <div>{data}</div>
            <div>{format}</div>
        </div>
    }
}

fn main() {
    // Demonstrate use of Rust `wasm-bindgen` https://rustwasm.github.io/docs/wasm-bindgen
    js::browser_js::run();
    wasm_logger::init(wasm_logger::Config::default());
    kobold::start(view! {
        <Editor />
    });
}
