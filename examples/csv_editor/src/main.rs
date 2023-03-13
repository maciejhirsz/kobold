use std::ops::Range;

use compact_str::CompactString;
use kobold::prelude::*;
use web_sys::HtmlInputElement;

mod csv;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

pub struct State {
    editing: Editing,
    name: String,
    table: Table,
}

pub struct Table {
    columns: Vec<CompactString>,
    rows: Vec<Vec<CompactString>>,
}

impl State {
    fn mock() -> Self {
        State {
            editing: Editing::None,
            name: "<no file>".to_owned(),
            table: Table::mock(),
        }
    }
}

impl Table {
    fn mock() -> Self {
        Table {
            columns: vec!["column 1".into(), "column 2".into()],
            rows: vec![
                vec!["A1".into(), "A2".into()],
                vec!["B1".into(), "B2".into()],
            ],
        }
    }

    fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}

#[component]
fn Editor() -> impl Html {
    stateful(State::mock, |state| {
        let onload = state.bind_async(move |hook, e: Event<HtmlInputElement>| async move {
            let file = match e.target().files().and_then(|list| list.get(0)) {
                Some(file) => file,
                None => return,
            };

            let _ = hook.update(|state| state.name = file.name());

            match csv::read_file(file).await {
                Ok(table) => {
                    let _ = hook.update(move |state| state.table = table);
                }
                Err(_) => (),
            };
        });

        html! {
            <input type="file" accept="text/csv" onchange={onload} />
            <h1>{ state.name.fast_diff() }</h1>
            <table>
                <thead>
                    <tr>
                    {
                        state.table.columns.iter().map(|c| html!{ <th>{ c.as_str() }</th> }).list()
                    }
                    </tr>
                </thead>
                <tbody>
                {
                    state.table.rows().map(move |row| html! {
                        <tr>
                        {
                            state.table.columns().map(move |col| html! {
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
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl Html + '_ {
    let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

    let onchange = state.bind(move |state, e: Event<HtmlInputElement>| {
        state.table.rows[row][col] = e.target().value().into();
        state.editing = Editing::None;
    });

    let value = state.table.rows[row][col].as_str();

    if state.editing == (Editing::Cell { row, col }) {
        html! {
            <td.edit>
                { value }
                <input.edit {onchange} value={ value.to_owned() } />
            </td>
        }
    } else {
        html! { <td {ondblclick}>{ value }</td> }
    }
}

fn main() {
    kobold::start(html! {
        <Editor />
    });
}
