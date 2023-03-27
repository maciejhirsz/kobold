// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::KoboldQR;
use web_sys::HtmlInputElement as InputElement;

mod csv;
mod state;

use state::{Editing, State, Text};

#[component]
fn Editor() -> impl Html {
    stateful(State::mock, |state| {
        let onload = state.bind_async(move |hook, e: Event<InputElement>| async move {
            let file = match e.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            let _ = hook.update(|state| state.name = file.name());

            if let Ok(table) = csv::read_file(file).await {
                let _ = hook.update(move |state| state.table = table);
            }
        });

        let onkeydown = state.bind(move |state, event: KeyboardEvent<_>| {
            if matches!(event.key().as_str(), "Esc" | "Escape") {
                state.editing = Editing::None;

                ShouldRender::Yes
            } else {
                ShouldRender::No
            }
        });

        html! {
            <div .invoice-wrapper>
                <section .invoiceapp>
                    <header .header>
                        <h1>"Tax Invoice"</h1>
                        <EntryInput {state} />
                    </header>
                    <section .main>
                        <input type="file" accept="text/csv" onchange={onload} />
                        <h1>{ state.name.fast_diff() }</h1>
                        <EntryView {state} />
                        <table {onkeydown}>
                            <thead>
                                <tr>
                                {
                                    state.columns().map(|col| html! { <Head {col} {state} /> }).list()
                                }
                                </tr>
                            </thead>
                            <tbody>
                            {
                                state.rows().map(move |row| html! {
                                    <tr>
                                    {
                                        state.columns().map(move |col| html! {
                                            <Cell {col} {row} {state} />
                                        })
                                        .list()
                                    }
                                    </tr>
                                })
                                .list()
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
fn Head(col: usize, state: &Hook<State>) -> impl Html + '_ {
    let value = state.source.get_text(&state.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.columns[col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        html! {
            <th.edit>
                { value.fast_diff() }
                <input.edit.edit-head {onchange} value={ value.fast_diff() } />
            </th>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Column { col });

        html! { <th {ondblclick}>{ value.fast_diff() }</th> }
    }
}

#[component(auto_branch)]
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl Html + '_ {
    let value = state.source.get_text(&state.rows[row][col]);

    if state.editing == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<InputElement>| {
            state.rows[row][col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        html! {
            <td.edit>
                { value }
                <input.edit {onchange} value={ value.to_owned() } />
            </td>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

        html! { <td {ondblclick}>{ value.fast_diff() }</td> }
    }
}

#[component]
fn EntryInput(state: &Hook<State>) -> impl Html + '_ {
    html! {
        <input
            .new-invoice
            placeholder="<Enter biller address>"
            onchange={state.bind(|state, event| {
                let input = event.target();
                let value = input.value();

                input.set_value("");
                state.add(value);
            })}
        />
    }
}

#[component]
fn EntryView<'a>(state: &'a Hook<State>) -> impl Html + 'a {
    let entry = &state.entry;
    let input = state.entry_editing.then(move || {
        let onmouseover = state.bind(move |_, event: MouseEvent<InputElement>| {
            let _ = event.target().focus();

            ShouldRender::No
        });

        let onkeypress = state.bind(move |state, event: KeyboardEvent<InputElement>| {
            if event.key() == "Enter" {
                state.update(event.target().value());

                ShouldRender::Yes
            } else {
                ShouldRender::No
            }
        });

        html! {
            <input .edit
                type="text"
                value={entry.description.fast_diff()}
                {onmouseover}
                {onkeypress}
                onblur={state.bind(move |state, event| state.update(event.target().value()))}
            />
        }
    });

    let editing = state.entry_editing.class("editing").no_diff();

    html! {
        <div .todo.{editing}>
            <div .view>
                <label ondblclick={state.bind(move |state, _| state.edit_entry())} >
                    { entry.description.fast_diff() }
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
            <KoboldQR data={data.as_str()} />
            <textarea {onkeyup}>
                { data.as_str().no_diff() }
            </textarea>
        }
    })
}

fn main() {
    kobold::start(html! {
        <Editor />
    });
}
