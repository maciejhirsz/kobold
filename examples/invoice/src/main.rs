// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;
use kobold::branching::{Branch2, Branch3, Empty};
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::KoboldQR;
use gloo_console::{console_dbg};
use gloo_utils::format::JsValueSerdeExt;
use gloo_file::{Blob, File as GlooFile};
use log::{info, debug, error, warn};
use serde::{Serialize, Deserialize};
use serde_json::{to_string};
use web_sys::{HtmlInputElement as InputElement, HtmlElement};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::throw_str;

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
            let file = match event.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            state.update_silent(|state| state.details.filename = file.name());

            if let Ok(table) = csv::read_file(file).await {
                debug!("details.table{:#?}", table);
                // https://docs.rs/kobold/latest/kobold/stateful/struct.Signal.html#method.update
                state.update_silent(move |state| {
                    state.details.table = table;
                    state.store(); // update local storage
                });
            }
        });

        let onsave_details = state.bind_async(|state, event: MouseEvent<HtmlElement>| async move {
            // update local storage and state so that &state.details isn't
            // `Content { filename: "\0\0\0\0\0\0\0", table: Table { source: TextSource { source: "\0" }, columns: [Insitu(0..0)], rows: [] } }`
            //
            // closure has access to Signal of state.
            // `update` doesn't implement Deref so you can't access fields on it like you can with a Hook
            // `update_silent` gives access to the actual state without triggering a render
            state.update_silent(|state| state.store());

            // closure required just to debug with access to state fields, since otherwise it'd trigger a render
            state.update_silent(|state| {
                debug!("onsave_details: {:?}", &state.details);
            });

            state.update_silent(|state| {
                match csv::generate_csv_data_obj_url_for_download(&state.details) {
                    Ok(obj_url) => {
                        debug!("obj_url {:?}", obj_url);

                        state.details.obj_url = obj_url;

                        // Automatically click the download button of the hyperlink with CSS id
                        // '#link-file-download' since the state should have been updated with the
                        // obj_url by now and that hyperlink has a `href` attribute that should
                        // now contain the obj_url that would be downloaded when that hyperlink is clicked
                        js::browser_js::run_click_element();
                    },
                    Err(err) => {
                        panic!("failed to generate csv data object url for download {:?}", state.details.filename);
                    },
                };
                debug!("successfully generate csv data object url for download {:?}", state.details.filename);
            });


        });

        let onload_main = state.bind_async(|state, event: Event<InputElement>| async move {
            let file = match event.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            state.update_silent(|state| state.main.filename = file.name());

            if let Ok(table) = csv::read_file(file).await {
                state.update_silent(move |state| {
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
                            // generates CSV file download object url and triggers the script __kobold_click_element.js that
                            // automatically clicks the #link-file-download hyperlink when the object url has been stored in state
                            <button #button-file-save type="button" onclick={onsave_details}>"Save to CSV file"</button><br />
                        </div>
                        <div>
                        {
                            if state.details.obj_url.len() > 0 && state.details.obj_url != "placeholder_url" {
                                Branch2::A(
                                    view! {
                                        // this link is hidden in the UI using CSS since it gets automatically clicked when
                                        // the download object url is saved in the state 
                                        <a #link-file-download href={ref state.details.obj_url}
                                            download={ref state.details.filename}>"Download CSV file to save changes"</a>
                                    }
                                )
                            } else {
                                Branch2::B(Empty)
                            }
                        }
                        </div>
                        // <EntryView {state} />
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
                                        state.editing = Editing::None;
                    
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
                    <p>"Hint: Double-click to edit an invoice field"</p>
                </footer>
            </div>
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.main.table.source.get_text(&state.main.table.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.main.table.columns[col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head
                    // TODO - is this required?
                    // onkeypress={
                    //     state.bind(move |state, e: KeyboardEvent<InputElement>| {
                    //         if e.key() == "Enter" && e.target().value() != "" {
                    //             state.update_main(row, col, e.target().value());

                    //             Then::Render
                    //         } else {
                    //             Then::Stop
                    //         }
                    //     })
                    // }

                    {onchange} value={ ref value }
                />
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
                    // TODO - is this required?
                    // onkeypress={
                    //     state.bind(move |state, e: KeyboardEvent<InputElement>| {
                    //         if e.key() == "Enter" && e.target().value() != "" {
                    //             state.update_main(row, col, e.target().value());

                    //             Then::Render
                    //         } else {
                    //             Then::Stop
                    //         }
                    //     })
                    // }
                    {onchange}
                    {onmouseenter}
                    value={ ref value }
                />
            </td>
        })
    // https://github.com/maciejhirsz/kobold/issues/51
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

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
    debug!("row/col: {:?}/{:?}", row, col);
    let value = state.details.table.source.get_text(&state.details.table.rows[row][col]);

    if state.editing_details == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.details.table.rows[row][col] = Text::Owned(e.target().value().into());
            state.editing_details = Editing::None;
        });

        view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head
                    // duplicate in CellDetails
                    // TODO - is this required?
                    // onkeypress={
                    //     state.bind(move |state, e: KeyboardEvent<InputElement>| {
                    //         if e.key() == "Enter" && e.target().value() != "" {
                    //             state.update_details(row, col, e.target().value());

                    //             Then::Render
                    //         } else {
                    //             Then::Stop
                    //         }
                    //     })
                    // }
                    {onchange} value={ ref value }
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
    let value = state.details.table.source.get_text(&state.details.table.rows[row][col]);

    if state.editing_details == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.details.table.rows[row][col] = Text::Owned(e.target().value().into());
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
                    // TODO - is this required?
                    // onkeypress={
                    //     state.bind(move |state, e: KeyboardEvent<InputElement>| {
                    //         if e.key() == "Enter" && e.target().value() != "" {
                    //             state.update_details(row, col, e.target().value());

                    //             Then::Render
                    //         } else {
                    //             Then::Stop
                    //         }
                    //     })
                    // }
                    {onchange} {onmouseenter} value={ ref value }
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
    // 
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
    debug!("main()");
    kobold::start(view! {
        <Editor />
    });
}
