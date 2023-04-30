use log::debug;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement};

use kobold::prelude::*;

use crate::js;
use crate::csv;
use crate::state::{Editing, State};
use crate::components::{
    Cell::{Cell},
    CellDetails::{CellDetails},
    Head::{Head},
    HeadDetails::{HeadDetails},
};

#[component]
pub fn Editor() -> impl View {
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
