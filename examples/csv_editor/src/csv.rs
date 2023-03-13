use std::ops::Range;
use std::str::FromStr;
use std::vec::IntoIter;

use logos::Logos;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

use crate::{Table, TextSource};

type Tokens = IntoIter<(Token, Range<usize>)>;

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

fn parse_row(
    lex: &mut Tokens,
    source: &mut Vec<u8>,
    columns: usize,
) -> Result<Option<Vec<Range<usize>>>, Error> {
    let mut row = Vec::with_capacity(columns);
    let mut value = None;

    while let Some((token, mut span)) = lex.next() {
        match token {
            Token::Value => value = Some(span),
            Token::QuotedValue => {
                span.start += 1;
                span.end -= 1;

                let mut slice = &mut source[span.clone()];

                while let Some(quote) = slice.windows(2).position(|w| w == b"\"\"") {
                    let len = slice.len();

                    slice.copy_within(quote + 1.., quote);
                    slice = &mut slice[quote + 1..len - 1];

                    span.end -= 1;
                }

                value = Some(span);
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
                row.resize_with(n, Range::default);
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
        let mut tokens = Token::lexer(&source)
            .spanned()
            .collect::<Vec<_>>()
            .into_iter();

        let mut source = source.into_bytes();

        let columns = parse_row(&mut tokens, &mut source, 0)?.ok_or(Error::NoData)?;

        let mut rows = Vec::new();

        while let Some(row) = parse_row(&mut tokens, &mut source, columns.len())? {
            rows.extend(row);
        }

        Ok(Table {
            source: TextSource { source },
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
