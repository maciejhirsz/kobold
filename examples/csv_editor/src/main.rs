use std::ops::Range;

use compact_str::CompactString;
use kobold::prelude::*;
use logos::{Lexer, Logos};
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, HtmlInputElement};

#[derive(Logos)]
enum Token {
    #[error]
    Err,
    #[token(",")]
    Comma,
    #[regex(r"[\n\r]+")]
    Newline,
    #[regex(r#"[^"\n\r,]+"#)]
    Value,
    #[regex(r#""([^"]|"")+""#)]
    QuotedValue,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

struct Table {
    editing: Editing,
    name: String,
    columns: Vec<CompactString>,
    rows: Vec<Vec<CompactString>>,
}

impl Table {
    fn mock() -> Self {
        Table {
            editing: Editing::None,
            name: "<no file>".to_owned(),
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

fn parse_row(lex: &mut Lexer<Token>) -> Option<Vec<CompactString>> {
    let mut row = Vec::new();
    let mut value = None;

    while let Some(token) = lex.next() {
        match token {
            Token::Value => value = Some(lex.slice().trim().into()),
            Token::QuotedValue => {
                let v = lex.slice();
                let v = &v[1..v.len() - 1];

                value = Some(v.replace("\"\"", "\"").into());
            }
            Token::Comma => {
                row.push(value.take().unwrap_or_default());
            }
            Token::Newline => {
                row.push(value.take().unwrap_or_default());
                break;
            }
            Token::Err => break,
        }
    }

    if row.is_empty() {
        None
    } else {
        Some(row)
    }
}

async fn read_file(file: File, hook: OwnedHook<Table>) {
    let text = match JsFuture::from(file.text()).await.map(|t| t.as_string()) {
        Ok(Some(text)) => text,
        _ => return,
    };

    let mut lex = Token::lexer(&text);

    let columns = parse_row(&mut lex).unwrap_or_default();

    let mut rows = Vec::new();

    while let Some(row) = parse_row(&mut lex) {
        rows.push(row);
    }

    hook.update(|table| {
        table.columns = columns;
        table.rows = rows;
    })
    .unwrap();
}

#[component]
fn Editor() -> impl Html {
    stateful(Table::mock, |table| {
        let onload = table.bind_async(move |hook, e: UntypedEvent<HtmlInputElement>| async move {
            let file = e.target().files().unwrap().get(0).unwrap();

            let _ = hook.update(|table| table.name = file.name());

            read_file(file, hook).await;
        });

        html! {
            <input type="file" accept="text/csv" onchange={onload} />
            <h1>{ table.name.fast_diff() }</h1>
            <table>
                <thead>
                    <tr>
                    {
                        table.columns.iter().map(|c| html!{ <th>{ c.as_str() }</th> }).list()
                    }
                    </tr>
                </thead>
                <tbody>
                {
                    table.rows().map(move |row| html! {
                        <tr>
                        {
                            table.columns().map(move |col| html! {
                                <Cell {col} {row} {table} />
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
fn Cell(col: usize, row: usize, table: &Hook<Table>) -> impl Html + '_ {
    let ondblclick = table.bind(move |t, _| t.editing = Editing::Cell { row, col });

    let onchange = table.bind(move |table, e: UntypedEvent<HtmlInputElement>| {
        table.rows[row][col] = e.target().value().into();
        table.editing = Editing::None;
    });

    let value = table.rows[row][col].as_str();

    if table.editing == (Editing::Cell { row, col }) {
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
