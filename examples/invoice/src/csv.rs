// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::{debug, error};
use logos::{Lexer, Logos};
use std::str::FromStr;
use take_mut::take;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, Url};

use crate::helpers::csv_helpers;
use crate::state::{Content, Table, TableVariant, Text, TextSource};

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[error]
    Err,
    // If we use `#` as the character before the TableVariant is mentioned
    // then we cannot use that character elsewhere in the Table `source` value
    #[regex(r#"[#].+"#, priority = 7)]
    Hash,
    #[token(",")]
    Comma,
    #[regex(r"[\n\r]+")]
    Newline,
    #[regex(r#"[^"\n\r,#]+"#)]
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
    FailedToReadFile,
    FailedToWriteFile,
    FailedToLoadMetadata,
    InvalidRowLength,
    MustBeAtLeastOneRowData,
    MustBeAtLeastOneColumnData,
    MustBeSameColumnLengthOnAllRows,
    MustBeTwoRowsIncludingLabelsRowDataRow,
    TableVariantUnsupported,
}

fn parse_table_variant(lex: &mut Lexer<Token>, columns: usize) -> Result<TableVariant, Error> {
    let mut table_variant = TableVariant::Unknown;

    while let Some(token) = lex.next() {
        match token {
            Token::Hash => {
                let mut slice = lex.slice();
                // lookahead and lookbehind are not supported `.+?(?=,)` or `.+?(,)`
                // so manually have to get value between @ and next comma

                // https://stackoverflow.com/a/37784410/3208553
                let start_bytes = slice.find("#").unwrap_or(0);
                let end_bytes = slice.find(",").unwrap_or(slice.len());
                let result = &slice[(start_bytes + 1)..end_bytes];
                debug!("parse_row result {:?}", result);
                table_variant = match result {
                    "main" => TableVariant::Main,
                    "details" => TableVariant::Details,
                    _ => return Err(Error::TableVariantUnsupported),
                };
            }
            Token::Value => continue,
            Token::QuotedValue => break,
            Token::EscapedValue => break,
            Token::Comma => break,
            Token::Newline => break,
            Token::Err => break,
            // allow users to not bother using a table variant
            _ => debug!("no table variant"),
        }

        if token == Token::Comma {
            break;
        }
    }
    Ok(table_variant)
}

fn parse_row(lex: &mut Lexer<Token>, columns: usize) -> Result<Option<Vec<Text>>, Error> {
    let mut row = Vec::with_capacity(columns);
    let mut value = None;

    while let Some(token) = lex.next() {
        let slice = lex.slice();
        debug!("slice {:?}", slice);

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
            _ => break,
        }
    }

    if let Some(value) = value {
        row.push(value);
    }

    debug!(
        "match columns row.len(), \n{:?}, \n{:?}",
        columns,
        row.len()
    );

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
        let mut table_variant = TableVariant::Unknown;
        let mut lex_just_to_get_variant = Token::lexer(&source);
        table_variant = match parse_table_variant(&mut lex_just_to_get_variant, 0) {
            Ok(variant) => variant,
            // FIXME - if i upload a file with prefix `#blah,...` it does not propagate and
            // show the error in the browser console
            // from parse_table_variant for some reason, it just stops execution
            // Err(err) => return Err(Error::TableVariantUnsupported),
            Err(err) => panic!("table variant unsupported"),
        };

        let mut trunc_source = source.clone();
        if table_variant != TableVariant::Unknown {
            debug!("table_variant {:?}", &table_variant);
            // we've obtained the `table_variant` from the file that's being uploaded
            // and we'll store that in the `Table` state, but if it
            // was not `TableVariant::Unknown` and it was a valid variant then we need
            // to remove it from the `source` so we'll create another version of it removed so
            // it doesn't interfere with processing the rest of the source

            let binding = trunc_source.find(",");
            let first_comma_index = match &binding {
                Some(idx) => idx,
                None => panic!("must be a comma after the table variant in source for it to exist"),
            };
            debug!("first_comma_index {:?}", &first_comma_index);
            trunc_source = (&trunc_source[first_comma_index + 1..]).to_string();
            debug!("trunc_source {:?}", &trunc_source);
        }

        // process without the truncated source
        let mut lex = Token::lexer(&trunc_source);

        let columns = parse_row(&mut lex, 0)?.ok_or(Error::NoData)?;
        debug!("columns {:?}", &columns);

        let mut rows = Vec::new();

        while let Some(row) = parse_row(&mut lex, columns.len())? {
            debug!("row {:?}", &row);
            rows.push(row);
        }
        debug!("rows {:?}", &rows);

        Ok(Table {
            variant: table_variant,
            source: TextSource::from(trunc_source),
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

fn validate_same_columns_length_all_rows(
    new_csv: &Vec<Vec<&str>>,
    new_csv_lens: &Vec<usize>,
) -> Result<(), Error> {
    let is_not_all_same =
        |new_csv: &[usize]| -> bool { new_csv.iter().min() != new_csv.iter().max() };
    debug!("is_not_all_same {:?}", is_not_all_same(&new_csv_lens));
    if is_not_all_same(&new_csv_lens) == true {
        return Err(Error::MustBeSameColumnLengthOnAllRows);
    }
    Ok(())
}

// the TableVariant::Main has a single label row, and then multiple data rows under it in the CSV file.
//
// it needs to be processed differently from TableVariant::Details that has only a single label row,
// a single data row.
pub fn generate_csv_data_for_download(content: &Content) -> Result<String, Error> {
    // generate CSV file format from object Url in state
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f911a069c22a7f4cf4b5e8a9aa05e65e
    let table_variant = &content.table.variant;

    match table_variant {
        TableVariant::Main => {
            let binding_source = &content.table.source.source;
            let original_csv: Vec<&str> = binding_source.split(&['\n'][..]).collect();
            debug!("original_csv {:?}", original_csv);

            let mut new_csv: Vec<Vec<&str>> = vec![];
            let mut new_csv_lens: Vec<usize> = vec![];
            let padding = "".to_string();
            csv_helpers::pad_csv_data(&original_csv, &mut new_csv, &mut new_csv_lens, &padding);

            debug!(
                "content.table.columns.len() {:?}",
                content.table.columns.len()
            );
            // validate qty of columns
            if content.table.columns.len() < 1 {
                return Err(Error::MustBeAtLeastOneColumnData);
            }
            debug!("new_csv_lens {:?}", new_csv_lens);

            validate_same_columns_length_all_rows(&new_csv, &new_csv_lens)?;

            let mut arr = vec![];
            let prefix = "#".to_string();
            let table_variant_str = "main".to_string();
            let prefix_table_variant: String = format!("{prefix}{table_variant_str},");

            // only one column so we'll process that first before the rows
            let new_csv_labels_stringified: String =
                update_csv_row_for_modified_table_cells(&content.table.columns, &mut new_csv[0]); // labels

            let combined = format!("{}{}", prefix_table_variant, new_csv_labels_stringified);
            arr.push(combined);
            debug!("arr combined: {:?}", arr);

            let content_table_rows = content.table.rows.clone();

            debug!("content_table_rows {:?}", content_table_rows);
            debug!("content_table_rows.len() {:?}", content_table_rows.len());
            // validate qty of rows
            if content_table_rows.len() < 1 {
                error!("{:?}", Error::MustBeAtLeastOneRowData);
                return Ok(arr.join("\n")); // Err(Error::MustBeAtLeastOneRowData);
            }

            // if the user add any additional rows to `content_table_rows`, then we need to add a
            // new row in the same index to new_csv.
            // since new_csv contains the label row, we subtract 1 from new_csv since we don't
            // want to include the label row in the length, since `content_table_rows` doesn't have
            // the labels row.
            let diff = match content_table_rows.len().checked_sub(new_csv.len() - 1) {
                Some(d) => d,
                None => 0,
            };
            if diff > 0 {
                for (i, row) in content_table_rows.iter().enumerate() {
                    // count how many elements in the row equal Owned("").
                    // if they're all `Owned("")` then that index was a row added by the user
                    let mut count_same: i32 = 0;
                    row.into_iter().enumerate().for_each(|(j, element)| {
                        match element {
                            Text::Owned(s) => {
                                if s.to_string() == "".to_string() {
                                    count_same += 1;
                                } else {
                                    // TODO - how to `continue` to next iteration in closure?
                                    // using something like `debug!("")` is dumb
                                    debug!(""); // continue
                                }
                            }
                            _ => debug!(""), // continue
                        }
                    });
                    // if they're all the same value of Owned("") on this row,
                    // then it was a row added to this index i by the user
                    if row.len() == count_same as usize {
                        if new_csv.get(i + 1).is_some() {
                            new_csv.insert(i + 1, vec!["", "", ""]);
                        } else {
                            // for rows added by the user to the end of
                            // `content_table_rows` that don't exist
                            // in `new_csv` then push them to the end
                            new_csv.push(vec!["", "", ""]);
                        }
                    }
                }
            }

            // debug!("new_csv {:?}", new_csv);
            // debug!("new_csv_lens {:?}", new_csv_lens);
            // debug!("content.table.rows {:?}", content.table.rows);

            // multiple rows so we'll push each of them now
            content_table_rows
                .into_iter()
                .enumerate()
                .for_each(|(i, row_data)| {
                    let new_csv_data_stringified: String = update_csv_row_for_modified_table_cells(
                        &content.table.rows[i],
                        // start with `+ 1` since `new_csv[0]` are its labels but we're only doing the rows
                        &mut new_csv[i + 1],
                    ); // values row 1
                    arr.push(new_csv_data_stringified);
                });

            let content_serialized: String = arr.join("\n");
            debug!("content_serialized {:?}", content_serialized);

            return Ok(content_serialized);
        }
        TableVariant::Details => {
            let binding_source = &content.table.source.source;
            let original_csv: Vec<&str> = binding_source.split(&['\n'][..]).collect();
            debug!("original_csv {:?}", original_csv);

            let mut new_csv: Vec<Vec<&str>> = vec![];
            let mut new_csv_lens: Vec<usize> = vec![];
            let padding = "".to_string();
            csv_helpers::pad_csv_data(&original_csv, &mut new_csv, &mut new_csv_lens, &padding);

            debug!(
                "content cols rows {:?} {:?}",
                content.table.columns.len(),
                content.table.rows.len()
            );
            // validate qty of rows
            if content.table.columns.len() != (1 as usize)
                && content.table.rows.len() != (1 as usize)
            {
                return Err(Error::MustBeTwoRowsIncludingLabelsRowDataRow);
            }

            debug!("new_csv_lens {:?}", new_csv_lens);

            validate_same_columns_length_all_rows(&new_csv, &new_csv_lens)?;

            let mut arr = vec![];
            let prefix = "#".to_string();
            let table_variant_str = "details".to_string();
            let prefix_table_variant: String = format!("{prefix}{table_variant_str},");

            let new_csv_labels_stringified: String =
                update_csv_row_for_modified_table_cells(&content.table.columns, &mut new_csv[0]);
            let combined = format!("{}{}", prefix_table_variant, new_csv_labels_stringified);
            arr.push(combined);
            debug!("arr combined: {:?}", arr);

            let new_csv_values_stringified: String =
                update_csv_row_for_modified_table_cells(&content.table.rows[0], &mut new_csv[1]);
            arr.push(new_csv_values_stringified);

            let content_serialized: String = arr.join("\n");
            debug!("content_serialized {:?}", content_serialized);

            return Ok(content_serialized);
        }
        TableVariant::Unknown => {
            return Err(Error::TableVariantUnsupported);
        }
        _ => {
            return Err(Error::TableVariantUnsupported);
        }
    };
}

pub fn update_csv_row_for_modified_table_cells<'a>(
    cells: &'a Vec<Text>,
    csv_row: &mut Vec<&'a str>,
) -> String {
    let _ = &cells.into_iter().enumerate().for_each(|(i, el)| {
        match el {
            Text::Insitu(r) => {}
            Text::Owned(s) => {
                // let len = csv_row.len() - 1;
                // https://users.rust-lang.org/t/replacing-element-of-vector/57258/3
                // use `take` so we have a closure that must return a valid T otherwise
                // the closure panics and program aborts incase it panics before we've
                // finished the process of swapping for the new value
                take(csv_row, |mut cr| {
                    // Note: Do not need this lengthy approach. Possibly don't need
                    // `take_mut` either.
                    // // removes elem at index i and swaps last elem into old index i
                    // let old_cell_data = &cr.swap_remove(i);
                    // cr.push(s); // push new elem to end of vector
                    // cr.swap(i, len); // swap new elem into index i
                    // debug!("replaced {:?} with {:?}", old_cell_data, s);

                    // Note: This is a simpler approach to replacing the value
                    core::mem::replace(&mut cr[i], s);
                    cr // must return valid T or it panics
                });
            }
        }
    });
    // debug!("{:?}", csv_row);
    let mut c = 0;
    let new_csv_data_stringified: String = csv_row
        .iter()
        .map(|text| {
            if c == csv_row.len() - 1 {
                c += 1;
                return text.to_string();
            }
            c += 1;
            return text.to_string() + ",";
        })
        .collect::<String>();
    new_csv_data_stringified
}
