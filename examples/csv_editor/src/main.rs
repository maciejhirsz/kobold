use kobold::prelude::*;
use web_sys::HtmlInputElement as Input;

mod csv;
mod state;

use state::{Editing, State, Text};

#[component]
fn Editor() -> impl View {
    stateful(State::mock, |state| {
        bind! {
            state:

            let onload = async |event: Event<Input>| {
                let file = match event.target().files().and_then(|list| list.get(0)) {
                    Some(file) => file,
                    None => return,
                };

                state.update(|state| state.name = file.name());

                if let Ok(table) = csv::read_file(file).await {
                    state.update(move |state| state.table = table);
                }
            };

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
            <input type="file" accept="text/csv" onchange={onload}>
            <h1>{ ref state.name }</h1>
            <table {onkeydown}>
                <thead>
                    <tr>
                    {
                        for state.columns().map(|col| view! { <Head {col} {state} /> })
                    }
                <tbody>
                {
                    for state.rows().map(move |row| view! {
                        <tr>
                        {
                            for state.columns().map(move |col| view! {
                                <Cell {col} {row} {state} />
                            })
                        }
                    })
                }
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.source.get_text(&state.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<Input>| {
            state.columns[col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        view! {
            <th.edit>
                { ref value }
                <input.edit.edit-head {onchange} value={ ref value }>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Column { col });

        view! { <th {ondblclick}>{ ref value } }
    }
}

#[component(auto_branch)]
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.source.get_text(&state.rows[row][col]);

    if state.editing == (Editing::Cell { row, col }) {
        bind! {
            state:

            let onchange = move |e: Event<Input>| {
                state.rows[row][col] = Text::Owned(e.target().value().into());
                state.editing = Editing::None;
            };
        }

        let mut selected = false;

        let onmouseenter = move |e: MouseEvent<Input>| {
            if !selected {
                let input = e.target();
                input.focus();
                input.select();
                selected = true;
            }
        };

        view! {
            <td.edit>
                { ref value }
                <input.edit {onchange} {onmouseenter} value={ ref value }>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

        view! { <td {ondblclick}>{ ref value } }
    }
}

fn main() {
    kobold::start(view! {
        <Editor />
    });
}
