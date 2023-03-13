use std::str::FromStr;

use compact_str::CompactString;
use logos::{Lexer, Logos};
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

use super::Table;

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

pub enum Error {
    NoData,
    FailedToReadFile,
    InvalidRowLength,
}

fn parse_row(lex: &mut Lexer<Token>, columns: usize) -> Result<Option<Vec<CompactString>>, Error> {
    let mut row = Vec::with_capacity(columns);
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

    const EMPTY: CompactString = CompactString::new_inline("");

    match (columns, row.len()) {
        (_, 0) => Ok(None),
        (0, _) => Ok(Some(row)),
        (n, r) => {
            if n > r {
                row.resize_with(n, || EMPTY);
            }

            if r > n {
                Err(Error::InvalidRowLength)
            } else {
                Ok(Some(row))
            }
        }
    }
}

impl FromStr for Table {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let mut lex = Token::lexer(s);

        let columns = parse_row(&mut lex, 0)?.ok_or(Error::NoData)?;

        let mut rows = Vec::new();

        while let Some(row) = parse_row(&mut lex, columns.len())? {
            rows.push(row);
        }

        Ok(Table { columns, rows })
    }
}

pub async fn read_file(file: File) -> Result<Table, Error> {
    let text = JsFuture::from(file.text())
        .await
        .map_err(|_| Error::FailedToReadFile)?
        .as_string()
        .ok_or(Error::FailedToReadFile)?;

    text.parse()
}
