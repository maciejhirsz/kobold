// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use logos::{Lexer, Logos};
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, Url};
use gloo_file::{Blob};
use take_mut::take;
use log::{debug};

use crate::state::{Content, Table, Text, TextSource};

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
    #[regex(r#""[^"]+""#, priority = 6)]
    QuotedValue,
    #[regex(r#""([^"]|"")+""#)]
    EscapedValue,
}

#[derive(Debug)]
pub enum Error {
    NoData,
    FailedToBufferFile,
    FailedToCreateObjectBlobWithUrl,
    FailedToReadFile,
    FailedToWriteFile,
    FailedToLoadMetadata,
    InvalidRowLength,
}

fn parse_row(lex: &mut Lexer<Token>, columns: usize) -> Result<Option<Vec<Text>>, Error> {
    let mut row = Vec::with_capacity(columns);
    let mut value = None;

    while let Some(token) = lex.next() {
        value = match token {
            Token::Value => Some(Text::Insitu(lex.span())),
            Token::QuotedValue => {
                let mut span = lex.span();

                span.start += 1;
                span.end -= 1;

                Some(Text::Insitu(span))
            }
            Token::EscapedValue => {
                let mut slice = lex.slice();

                slice = &slice[1..slice.len() - 1];

                Some(Text::Owned(slice.replace("\"\"", "\"").into()))
            }
            Token::Comma => {
                row.push(value.take().unwrap_or_default());
                continue;
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

pub async fn generate_csv_data_obj_url_for_download(content: &Content) -> Result<String, Error> {
    // generate CSV file format from object Url in state
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f911a069c22a7f4cf4b5e8a9aa05e65e

    let binding_source = &content.table.source.source;
    let original_csv: Vec<&str> = binding_source.split(&['\n'][..]).collect();
    debug!("original_csv {:?}", original_csv);
    let old_csv: Vec<Vec<&str>> = vec![
        original_csv[0].split(",").collect(), // variable of each label
        original_csv[1].split(",").collect(), // values of each label
        original_csv[2].split(",").collect(), // label
    ];
    let mut new_csv: Vec<Vec<&str>> =
        vec![old_csv[0].clone(), old_csv[1].clone(), old_csv[2].clone()];

    let new_csv_variables_stringified: String = update_csv_row_for_modified_table_cells(&content.table.columns, &mut new_csv[0]);
    let new_csv_values_stringified: String = update_csv_row_for_modified_table_cells(&content.table.rows[0], &mut new_csv[1]);
    let new_csv_labels_stringified: String = update_csv_row_for_modified_table_cells(&content.table.rows[1], &mut new_csv[2]);
    let arr = vec![new_csv_variables_stringified, new_csv_values_stringified, new_csv_labels_stringified];
    // println!("{:?}", arr);
    let content_serialized: String = arr.join("\n");
    debug!("content_serialized {:?}", content_serialized);

    // cast String into a byte slice
    let content_serialized_byte_slice: &[u8] = &content_serialized.as_bytes();

    let file_blob: Blob = Blob::new_with_options(
        content_serialized_byte_slice,
        Some("text/plain"),
    );
    debug!("file_blob: {:?}", file_blob);
    // convert struct `gloo_file::Blob` into struct `web_sys::Blob`
    let obj_url = match Url::create_object_url_with_blob(&file_blob.into()) {
        Ok(url) => url,
        Err(err) => return Err(Error::FailedToCreateObjectBlobWithUrl),
    };

    return Ok(obj_url);
}

pub fn update_csv_row_for_modified_table_cells<'a>(
    cells: &'a Vec<Text>,
    csv_row: &mut Vec<&'a str>
) -> String {
    let _ = &cells
        .into_iter()
        .enumerate()
        .for_each(|(i, el)| {
            match el {
                Text::Insitu(r) => {},
                Text::Owned(s) => {
                    let len = csv_row.len() - 1;
                    // https://users.rust-lang.org/t/replacing-element-of-vector/57258/3
                    // use `take` so we have a closure that must return a valid T otherwise the closure panics and program aborts
                    // incase it panics before we've finished the process of swapping for the new value
                    take(csv_row, |mut cr| {
                        // Note: Do not need this lengthy approach. Possibly don't need `take_mut` either
                        // let old_cell_data = &cr.swap_remove(i); // removes elem at index i and swaps last elem into old index i
                        // cr.push(s); // push new elem to end of vector
                        // cr.swap(i, len); // swap new elem into index i
                        // debug!("replaced {:?} with {:?}", old_cell_data, s);
    
                        // Note: This is a simpler approach to replacing the value
                        core::mem::replace(&mut cr[i], s);
                        cr // must return valid T or it panics
                    });  
                },
            }
        });
    // println!("{:?}", csv_row);
    let mut c = 0;
    let new_csv_variables_stringified: String =
        csv_row.iter().map(|text| {
            if c == csv_row.len() - 1 {
                c += 1;
                return text.to_string();
            }
            c += 1;
            return text.to_string() + ",";
        }).collect::<String>();
    new_csv_variables_stringified
}