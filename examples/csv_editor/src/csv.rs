use std::str::FromStr;

use logos::{Lexer, Logos};
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

use crate::state::{Table, Text, TextSource};

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

#[derive(Debug)]
pub enum Error {
    NoData,
    FailedToReadFile,
    InvalidRowLength,
}

fn parse_row(lex: &mut Lexer<Token>, columns: usize) -> Result<Option<Vec<Text>>, Error> {
    let mut row = Vec::with_capacity(columns);
    let mut value = None;

    while let Some(token) = lex.next() {
        match token {
            Token::Value => value = Some(Text::Insitu(lex.span())),
            Token::QuotedValue => {
                value = Some(Text::Owned(lex.slice().replace("\"\"", "\"").into()));
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

    if let Some(value) = value {
        row.push(value);
    }

    match (columns, row.len()) {
        (_, 0) => Ok(None),
        (0, _) => Ok(Some(row)),
        (n, r) => {
            if n > r {
                row.resize_with(n, Text::default);
            }

            if r > n {
                Err(Error::InvalidRowLength)
            } else {
                Ok(Some(row))
            }
        }
    }
}

impl TryFrom<String> for Table {
    type Error = Error;

    fn try_from(source: String) -> Result<Self, Error> {
        let mut lex = Token::lexer(&source);

        let columns = parse_row(&mut lex, 0)?.ok_or(Error::NoData)?;

        let mut rows = Vec::new();

        while let Some(row) = parse_row(&mut lex, columns.len())? {
            rows.push(row);
        }

        Ok(Table {
            source: TextSource::from(source),
            columns,
            rows,
        })
    }
}

impl FromStr for Table {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        s.to_owned().try_into()
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
