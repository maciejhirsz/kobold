use kobold::prelude::*;
use web_sys::HtmlInputElement as Input;
use wasm_bindgen_futures::spawn_local;

mod csv;
mod state;

use state::{Editing, State, Text};

#[component]
fn Editor() -> impl Html {
    stateful(State::mock, |state| {
        let onload = {
            let signal = state.signal();

            move |e: Event<Input>| {
                let file = match e.target().files().and_then(|list| list.get(0)) {
                    Some(file) => file,
                    None => return,
                };

                signal.update(|state| state.name = file.name());

                let signal = signal.clone();

                spawn_local(async move {
                    if let Ok(table) = csv::read_file(file).await {
                        signal.update(move |state| state.table = table);
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

        html! {
            <input type="file" accept="text/csv" onchange={onload} />
            <h1>{ state.name.fast_diff() }</h1>
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
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, state: &Hook<State>) -> impl Html + '_ {
    let value = state.source.get_text(&state.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<Input>| {
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
        let onchange = state.bind(move |state, e: Event<Input>| {
            state.rows[row][col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        html! {
            <td.edit>
                { value.fast_diff() }
                <input.edit {onchange} value={ value.fast_diff() } />
            </td>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

        html! { <td {ondblclick}>{ value.fast_diff() }</td> }
    }
}

fn main() {
    kobold::start(html! {
        <Editor />
    });
}
