use log::debug;
use std::ops::Deref;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement, UiEvent};

use kobold::prelude::*;

use crate::components::{
    Cell::Cell, CellDetails::CellDetails, Head::Head, HeadDetails::HeadDetails,
};
use crate::csv;
use crate::js;
use crate::state::{Editing, State, TableVariants};

async fn onload_common(
    table_variant: TableVariants,
    state: Signal<State>,
    event: Event<InputElement>,
) {
    debug!("onload_details");
    let file = match event.target().files().and_then(|list| list.get(0)) {
        Some(file) => file,
        None => return,
    };

    // TODO - should only update filename if the upload in the next step was successful
    state.update(|state| {
        match table_variant {
            TableVariants::Main => state.main.filename = file.name(),
            TableVariants::Details => state.details.filename = file.name(),
            _ => panic!("unknown variant name to upload table with filename"),
        };
    });

    if let Ok(table) = csv::read_file(file).await {
        debug!("table {:#?}", table);
        // https://docs.rs/kobold/latest/kobold/stateful/struct.Signal.html#method.update
        state.update(move |state| {
            match table_variant {
                TableVariants::Main => {
                    state.main.table = table;
                }
                TableVariants::Details => {
                    state.details.table = table;
                }
                _ => panic!("unknown variant name to upload table"),
            };
            state.store(); // update local storage
        });
    }
}

async fn onsave_common(
    table_variant: TableVariants,
    state: Signal<State>,
    event: MouseEvent<HtmlElement>,
) {
    // closure required just to debug with access to state fields, since otherwise it'd trigger a render
    state.update_silent(|state| {
        debug!("onsave_details: {:?}", &state.details);
        debug!("onsave_main: {:?}", &state.main);
    });

    state.update(|state| {
        // update local storage and state so that &state.details isn't
        // `Content { filename: "\0\0\0\0\0\0\0", table: Table { source: TextSource
        //   { source: "\0" }, columns: [Insitu(0..0)], rows: [] } }`
        state.store();

        match table_variant {
            TableVariants::Main => {
                // only setup for details
                match csv::generate_csv_data_for_download(TableVariants::Main, &state.main) {
                    Ok(csv_data) => {
                        debug!("csv_data {:?}", csv_data);
                        // cast String into a byte slice
                        let csv_data_byte_slice: &[u8] = csv_data.as_bytes();
                        js::browser_js::run_save_file(&state.main.filename, csv_data_byte_slice);
                    }
                    Err(err) => {
                        panic!("failed to generate csv data for download: {:?}", err);
                    }
                };
                debug!("successfully generated csv data for download");
            }
            TableVariants::Details => {
                match csv::generate_csv_data_for_download(TableVariants::Details, &state.details) {
                    Ok(csv_data) => {
                        debug!("csv_data {:?}", csv_data);
                        // cast String into a byte slice
                        let csv_data_byte_slice: &[u8] = csv_data.as_bytes();
                        js::browser_js::run_save_file(&state.details.filename, csv_data_byte_slice);
                    }
                    Err(err) => {
                        panic!("failed to generate csv data for download: {:?}", err);
                    }
                };
                debug!("successfully generated csv data for download");
            }
            _ => panic!("unknown variant name to save csv data"),
        };
    });
}

#[component]
pub fn Editor() -> impl View {
    stateful(State::default, |state| {
        debug!("Editor()");

        let onload_details = state.bind_async(|state, event: Event<InputElement>| async move {
            onload_common(TableVariants::Details, state, event).await;
        });

        let onload_main = state.bind_async(|state, event: Event<InputElement>| async move {
            onload_common(TableVariants::Main, state, event).await;
        });

        let onsave_details = state.bind_async(|state, event: MouseEvent<HtmlElement>| async move {
            onsave_common(TableVariants::Details, state, event).await;
        });

        let onsave_main = state.bind_async(|state, event: MouseEvent<HtmlElement>| async move {
            onsave_common(TableVariants::Main, state, event).await;
        });

        view! {
            <div .invoice-wrapper>
                <section .invoiceapp>
                    <header .header>
                        <h1>"Invoice"</h1>
                    </header>
                    <section .main>
                        <h3>"Details table"</h3>
                        <div class="container">
                            // https://stackoverflow.com/a/48499451/3208553
                            <input type="file" class="file-input-hidden" id="file-input-details" accept="text/csv" onchange={onload_details} onclick="this.value=null;" />
                            <input type="button" id="file-input-details-modern" onclick="document.getElementById('file-input-details').click()" value="Upload CSV file (Details)" />
                            <label for="file-input-details" class="label">{ ref state.details.filename }</label>
                            <button #button-file-save type="button" onclick={onsave_details}>"Save to CSV file"</button>
                        </div>
                        <br /><br />
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
                    </section>
                    <section .main>
                        <h3>"Main table"</h3>
                        <div class="container">
                            <input type="file" class="file-input-hidden" id="file-input-main" accept="text/csv" onchange={onload_main} onclick="this.value=null;" />
                            <input type="button" id="file-input-main-modern" onclick="document.getElementById('file-input-main').click()" value="Upload CSV file (Main)" />
                            <label for="file-input-main" class="label">{ ref state.main.filename }</label>
                            <button #button-file-save type="button" onclick={onsave_main}>"Save to CSV file"</button>
                        </div>
                        <br /><br />
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
