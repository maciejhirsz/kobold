// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
// use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
// use serde_json::{from_str, to_string};
use log::{info, debug, error, warn};
use std::convert::TryInto;

use std::ops::{Deref, DerefMut, Range};

const KEY_MAIN: &str = "kobold.invoice.main";
const KEY_DETAILS: &str = "kobold.invoice.details";

#[derive(Deserialize, Debug)]
pub enum Error {
    StorageError,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub filename: String,
    pub obj_url: String,
    pub table: Table,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub editing_main: Editing,
    pub editing_details: Editing,
    pub main: Content,
    pub details: Content,
}

/// - all `Table` cells should be populated with `Insitu` by default, the only exception is when you
/// have escapes in the loaded CSV. e.g. if your CSV contains quotes in quotes, the parser needs
/// to change escapes quotes into unescaped ones, so it will allocate a String to do it in. for 
/// a value in quotes it slices with +1/-1 to skip quotes, and then for escapes it also skips
/// quotes and then replaces escaped quotes inside. if you put something like: `"hello ""world"""`
/// in your CSV file, that will be `Text::Owned`
/// - the `Table` `source` property values should be read only
/// - if you edit a `Table` cell, just swap it from `Insitu` to `Owned` text
/// - you get an owned string from `.value()` so there is no point in trying to avoid it
/// - loading a file prefers `Insitu` since it can just borrow all unescaped values
/// from the `source` without allocations
/// - it uses `fn parse_row` in csv.rs to magically know whether to store in `Insitu`
/// instead of `Owned`, otherwise we explicitly tell it to use `Insitu` when setting
/// the default value `Text::Insitu(0..0)` in this file and when we edit a field
/// in the UI so it becomes `Owned("text")` (where text is what we enter)
/// - credit: Maciej
#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub source: TextSource,
    pub columns: Vec<Text>,
    pub rows: Vec<Vec<Text>>,
}

/// Text is used instead of just String to avoid unnecessary allocations that are expensive, since
/// subslicing the `source` with an `Insitu` `range` is a const operation, so it's just fiddling with
/// a pointer and the length - so it's not exactly free, but it's as close to free as you can get.
/// even better would be for `Insitu` to contain `&str`, but internal borrowing is a bit of a pain
/// - credit: Maciej
#[derive(Serialize, Deserialize, Debug)]
pub enum Text {
    Insitu(Range<usize>),
    Owned(Box<str>),
}

impl Default for Text {
    fn default() -> Self {
        Text::Insitu(0..0)
    }
}

impl Default for State {
    fn default() -> Self {
        let main_local_storage: Table = match LocalStorage::get(KEY_MAIN) {
            Ok(local_storage) => local_storage,
            Err(err) => Table::mock(),
        };
        let details_local_storage: Table = match LocalStorage::get(KEY_DETAILS) {
            Ok(local_storage) => local_storage,
            // TODO - check that this actually converts to Table type
            Err(err) => Table::mock_file_details(),
        };
        debug!("loading local storage: {:?}\n\n{:?}", main_local_storage, details_local_storage);

        State {
            editing_main: Editing::None,
            editing_details: Editing::None,
            main: Content {
                filename: "main.csv".to_owned(),
                obj_url: "placeholder_url".to_owned(),
                table: main_local_storage,
            },
            details: Content {
                filename: "details.csv".to_owned(),
                obj_url: "placeholder_url".to_owned(),
                table: details_local_storage,
            },
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing_main: Editing::None,
            editing_details: Editing::None,
            main: Content {
                filename: "main.csv".to_owned(),
                obj_url: "placeholder_url".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                filename: "details.csv".to_owned(),
                obj_url: "placeholder_url".to_owned(),
                table: Table::mock_file_details(),   
            },
        }
    }

    #[inline(never)]
    // store the updated state in web browser local storage
    pub fn store(&self) {
        debug!("updating store: {:?}\n\n{:?}", &self.main.table, &self.details.table);
        LocalStorage::set(KEY_MAIN, &self.main.table).unwrap_throw();
        LocalStorage::set(KEY_DETAILS, &self.details.table).unwrap_throw();
    }

    // // get specific storage of 'details' key
    // pub fn get_store_details(&self) -> Result<Table, Error> {
    //     debug!("get_store_details");

    //     let details_local_storage: Table = match LocalStorage::get(KEY_DETAILS) {
    //         Ok(local_storage) => {
    //             debug!("local_storage {:?}", local_storage);
    //             local_storage
    //         },
    //         Err(err) => {
    //             debug!("err {:?}", err);
    //             return Err(Error::StorageError);
    //         },
    //     };
    //     Ok(details_local_storage)
    // }

    // store in state edits by user to the 'main' table of the UI
    pub fn update_main(&mut self, row: usize, col: usize, value: String) {
        let old_storage = self.main.table.source.get_text(&self.main.table.rows[row][col]);
        if value != old_storage {
            self.main.table.rows[row][col] = Text::Owned(value.into());
            self.editing_main = Editing::None; // also done in onkeydown
            self.store();
        }
    }

    // store in state edits by user to the 'details' table of the UI
    pub fn update_details(&mut self, row: usize, col: usize, value: String) {
        debug!("update_details: {:?}\n{:?}\n{:?}", row, col, value);
        let old_storage = self.details.table.source.get_text(&self.details.table.rows[row][col]);
        debug!("update_details old storage: {:?}", old_storage);
        if value != old_storage {
            self.details.table.rows[row][col] = Text::Owned(value.into());
            self.editing_details = Editing::None; // also done in onkeydown

            self.store();
        }
    }
}

impl Deref for Content {
    type Target = Table;

    fn deref(&self) -> &Table {
        &self.table
    }
}

impl DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextSource {
    pub source: String,
}

impl From<String> for TextSource {
    fn from(source: String) -> Self {
        TextSource { source }
    }
}

impl TextSource {
    pub fn get_text<'a>(&'a self, text: &'a Text) -> &'a str {
        match text {
            Text::Insitu(span) => &self.source[span.clone()],
            Text::Owned(string) => string,
        }
    }
}

impl Table {
    fn mock() -> Self {
        "description,total,qr\ntask1,10,0x000|h160\ntask2,20,0x100|h160".parse().unwrap_throw()
    }

    fn mock_file_details() -> Self {
        "inv_date,inv_no,from_attn_name,from_org_name,from_org_addr,from_email,to_attn_name,to_title,to_org_name,to_email\n01.04.2023,0001,luke,clawbird,1 metaverse ave,test@test.com,recipient_name,director,nftverse,test2@test.com\ninvoice date,invoice number,name person from,organisation name from,organisation address from,email from,name person attention to,title to,organisation name to,email to".parse().unwrap_throw()
    }

    pub fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    pub fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}
