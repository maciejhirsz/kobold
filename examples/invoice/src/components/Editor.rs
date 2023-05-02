use log::debug;
use std::ops::Deref;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement as InputElement, UiEvent};

use kobold::prelude::*;

use crate::components::{
    Cell::Cell, CellDetails::CellDetails, Head::Head, HeadDetails::HeadDetails,
};
use crate::csv;
use crate::js;
use crate::state::{Content, Editing, State, TableVariant};

// running `get(state)` returns either `state.main` or `state.details`
async fn onload_common(
    table_variant: TableVariant,
    get: impl Fn(&mut State) -> &mut Content,
    state: Signal<State>,
    event: Event<InputElement>,
) {
    debug!("onload_common");

    let file = match event.target().files().and_then(|list| list.get(0)) {
        Some(file) => file,
        None => return,
    };

    event.target().set_value("");

    // TODO - should only update filename if the upload in the next step was successful
    state.update(|state| get(state).filename = file.name());

    if let Ok(table) = csv::read_file(file).await {
        debug!("table {:#?}", &table);
        debug!("table_variant {:#?}", &table_variant);

        // https://docs.rs/kobold/latest/kobold/stateful/struct.Signal.html#method.update
        state.update(move |state| {
            get(state).table = table;
            // TODO - might not be necessary if handled in csv.rs `try_from`
            get(state).table.variant = table_variant;
            state.store(); // update local storage
        });
    }
}

// TODO - we shouldn't need to pass `TableVariant` as a parameter since anything we've uploaded to be saved
// should already have a variant associated with it, otherwise it'll be processed as `TableVariant::Unknown`.
// so we need to prefix the saved file's stringified value with its variant (e.g. `#details,\n...`),
// in the function `generate_csv_data_for_download`
// so it's ready to be processed if they re-uploaded it again later.
// See draft code here https://github.com/maciejhirsz/kobold/commit/9287489c8091eb4c435940394bef1e1faa0da046#diff-466748d62629a0b88b2cc503a3d905976f38734933f05ae4032fe5f2b06bd2f4R42
// and here https://github.com/maciejhirsz/kobold/commit/9287489c8091eb4c435940394bef1e1faa0da046#diff-e5b6bd12f72bc9b411526063b4215b05bf5e7686f083cb2269685fe73886c7b6R273
async fn onsave_common(
    table_variant: TableVariant,
    get: impl Fn(&mut State) -> &mut Content,
    state: Signal<State>,
    event: MouseEvent<HtmlElement>,
) {
    // closure required just to debug with access to state fields, since otherwise it'd trigger a render
    state.update_silent(|state| debug!("onsave_: {:?}", &get(state)));

    state.update(|state| {
        // update local storage and state so that &state.details isn't
        // `Content { filename: "\0\0\0\0\0\0\0", table: Table { source: TextSource
        //   { source: "\0" }, columns: [Insitu(0..0)], rows: [] } }`
        state.store();

        match table_variant {
            TableVariant::Main => {
                // only setup for details
                match csv::generate_csv_data_for_download(TableVariant::Main, &get(state)) {
                    Ok(csv_data) => {
                        debug!("csv_data {:?}", csv_data);
                        // cast String into a byte slice
                        let csv_data_byte_slice: &[u8] = csv_data.as_bytes();
                        js::browser_js::run_save_file(&get(state).filename, csv_data_byte_slice);
                    }
                    Err(err) => {
                        panic!("failed to generate csv data for download: {:?}", err);
                    }
                };
                debug!("successfully generated csv data for download");
            }
            TableVariant::Details => {
                match csv::generate_csv_data_for_download(TableVariant::Details, &get(state)) {
                    Ok(csv_data) => {
                        debug!("csv_data {:?}", csv_data);
                        // cast String into a byte slice
                        let csv_data_byte_slice: &[u8] = csv_data.as_bytes();
                        js::browser_js::run_save_file(&get(state).filename, csv_data_byte_slice);
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

        // "closure needs to return the future onload_common returns for async_bind to work" - Maciej
        let onload_details = state.bind_async(|state, event: Event<InputElement>| {
            debug!("onload_details");
            onload_common(
                TableVariant::Details,
                |state| &mut state.details,
                state,
                event,
            )
        });

        let onload_main = state.bind_async(|state, event: Event<InputElement>| {
            onload_common(TableVariant::Main, |state| &mut state.main, state, event)
        });

        let onsave_details = state.bind_async(|state, event: MouseEvent<HtmlElement>| {
            onsave_common(
                TableVariant::Details,
                |state| &mut state.details,
                state,
                event,
            )
        });

        let onsave_main = state.bind_async(|state, event: MouseEvent<HtmlElement>| {
            onsave_common(TableVariant::Main, |state| &mut state.main, state, event)
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
                            <input type="file" class="file-input-hidden" id="file-input-details" accept="text/csv" onchange={onload_details} />
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
                            <input type="file" class="file-input-hidden" id="file-input-main" accept="text/csv" onchange={onload_main} />
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
