use std::ops::Range;

use kobold::prelude::*;
use web_sys::HtmlInputElement;

mod csv;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Editing {
    None,
    Cell(usize),
}

pub struct State {
    editing: Editing,
    name: String,
    table: Table,
}

pub struct Table {
    source: TextSource,
    columns: Vec<Range<usize>>,
    rows: Vec<Range<usize>>,
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

pub struct TextSource {
    source: Vec<u8>,
}

impl From<String> for TextSource {
    fn from(value: String) -> Self {
        TextSource { source: value.into_bytes() }
    }
}

impl TextSource {
    pub fn get_text(&self, span: &Range<usize>) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.source[span.clone()]) }
    }

    pub fn update_text(&mut self, span: &mut Range<usize>, new: &str) {
        *span = if new.len() <= span.end - span.start {
            let new_span = span.start .. span.start + new.len();
            self.source[new_span.clone()].copy_from_slice(new.as_bytes());

            new_span
        } else {
            self.push(new)
        };
    }

    pub fn push(&mut self, slice: &str) -> Range<usize> {
        let new_span = self.source.len() .. self.source.len() + slice.len();

        self.source.extend_from_slice(slice.as_bytes());

        new_span
    }
}

impl Table {
    fn mock() -> Self {
        "column 1,column 2\nA1,A2\nB1,B2".parse().unwrap()
    }

    fn rows(&self) -> impl Iterator<Item = usize> {
        (0..self.rows.len()).step_by(self.columns.len())
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

        let table = &state.table;

        html! {
            <input type="file" accept="text/csv" onchange={onload} />
            <h1>{ state.name.fast_diff() }</h1>
            <table>
                <thead>
                    <tr>
                    {
                        table.columns.iter().map(|c| html!{ <th>{ table.source.get_text(c) }</th> }).list()
                    }
                    </tr>
                </thead>
                <tbody>
                {
                    state.table.rows().map(move |row| html! {
                        <tr>
                        {
                            table.columns().map(move |col| html! {
                                <Cell cell={col + row} {state} />
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
fn Cell(cell: usize, state: &Hook<State>) -> impl Html + '_ {
    let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell(cell));

    let onchange = state.bind(move |state, e: Event<HtmlInputElement>| {
        let span = &mut state.table.rows[cell];

        state.table.source.update_text(span, &e.target().value());
        state.editing = Editing::None;
    });

    let value = state.table.source.get_text(&state.table.rows[cell]);

    if state.editing == (Editing::Cell(cell)) {
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
