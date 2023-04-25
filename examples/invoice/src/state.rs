// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
// use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
use serde_json::{from_str, to_string};
use log::{info, debug, error, warn};
use std::convert::TryInto;

use std::ops::{Deref, DerefMut, Range};

const KEY_MAIN: &str = "kobold.invoice.main";
const KEY_DETAILS: &str = "kobold.invoice.details";

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub name: String,
    pub table: Table,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub editing: Editing,
    pub editing_details: Editing,
    pub main: Content,
    pub details: Content,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub source: TextSource,
    pub columns: Vec<Text>,
    pub rows: Vec<Vec<Text>>,
}

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
        let mut default_data = "_,_,_,_";
        let mut storage = format!("{:#?}", default_data);

        LocalStorage::raw().set_item(KEY_MAIN, &storage).ok();
        LocalStorage::raw().set_item(KEY_DETAILS, &storage).ok();

        State {
            editing: Editing::None,
            editing_details: Editing::None,
            main: Content {
                name: "main".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "details".to_owned(),
                table: Table::mock_file_details(),   
            },
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing: Editing::None,
            editing_details: Editing::None,
            main: Content {
                name: "main".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "details".to_owned(),
                table: Table::mock_file_details(),   
            },
        }
    }

    #[inline(never)]
    // store the updated state in web browser local storage
    pub fn store(&self) {
        let main_str = to_string(&self.main.table).unwrap();
        let details_str = to_string(&self.details.table).unwrap();
        debug!("updating store: {:?}\n\n{:?}", main_str.as_str(), details_str.as_str());
        LocalStorage::raw().set_item(KEY_MAIN, main_str.as_str()).ok();
        LocalStorage::raw().set_item(KEY_DETAILS, details_str.as_str()).ok();
    }

    // store in state edits by user to the 'main' table of the UI
    pub fn update_main(&mut self, row: usize, col: usize, value: String) {
        let old_storage = self.main.table.source.get_text(&self.main.table.rows[row][col]);
        if value != old_storage {
            self.store();
        }

        // TODO - update this similar to how i've updated `update_details` method
    }

    // store in state edits by user to the 'details' table of the UI
    pub fn update_details(&mut self, row: usize, col: usize, value: String) {
        debug!("update_details: {:?}\n{:?}\n{:?}", row, col, value);
        let old_storage = self.details.table.source.get_text(&self.details.table.rows[row][col]);
        debug!("update_details old storage: {:?}", old_storage);
        if value != old_storage {
            debug!("updating details since different");
            self.details.table.rows[row][col] = Text::Owned(value.into());
            // TODO - convert old_storage to an object variable

            let serialized = serde_json::to_string(&self.details.table).unwrap();
            let value = JsValue::from_serde(&serialized).unwrap();
            debug!("details table {:#?}", value);
            let details_table: Table = serde_json::from_str(&serialized).unwrap();
            debug!("details table source {:#?}", &details_table.source.source);
            let data: &str = &details_table.source.source.to_string();


            let count_newlines = data.matches("\n").count();
            let (row1_idx_end, _) = data.match_indices("\n").nth(0).unwrap();
            let row0_idx_start = row1_idx_end + 1; // where +1 is to skip the `\n`
            let (row0_idx_end, _) = data.match_indices("\n").nth(1).unwrap();
            let row2_idx_start = row0_idx_end + 1;
            let row2_idx_end = data.len() - count_newlines;
            debug!("row1_idx_end {:#?}", row1_idx_end); // variables
            debug!("row0_idx_end {:#?}", row0_idx_end); // labels
            debug!("row2_idx_end {:#?}", row2_idx_end); // data

            let mut row0_vec: Vec<String> = Vec::new();
            let row0 = &data[row0_idx_start..row0_idx_end];
            row0_vec = row0.split(",").map(|x| x.to_string()).collect();
            debug!("row0_vec {:#?}", row0_vec);

            let mut row2_vec: Vec<String> = Vec::new();
            let row2 = &data[row2_idx_start..row2_idx_end];
            row2_vec = row2.split(",").map(|x| x.to_string()).collect();
            debug!("row2_vec {:#?}", row2_vec);

            // let blank_str = "".to_string();
            if row == 0 {
                debug!("updating row0_vec[col] {:#?}", row0_vec[col]);
                // https://docs.rs/wasm-bindgen/0.2.84/wasm_bindgen/struct.JsValue.html#method.as_string
                row0_vec[col] = match value.as_string() {
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                };
            } else if row == 1 { // need row variable to be 1 to update row data 2
                debug!("updating row2_vec[col] {:#?}", row2_vec[col]);
                row2_vec[col] = match value.as_string() {
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                };
            } else {
                panic!("cannot update this row from the ui"); // `row` value of `2` won't be provided to this fn
            }

            // check that the table source at `rows[row][col]` has been updated with the new value
            let updated_old_storage = self.details.table.source.get_text(&self.details.table.rows[row][col]);
            debug!("update_details updated_old_storage: {:?}", updated_old_storage);

            // TODO - whilst it is being updated in the above logs, when we later call `self.store()`
            // the value of the data that we retrieve from `self` in that method is different when we run
            // `to_string(&self.details.table).unwrap()`, so changes to `self` in this method aren't being
            // reflected in the `self.store()` method

            // TODO - we're already doing this in main.rs onkeydown so maybe we don't need it here too
            self.editing_details = Editing::None;

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
        "description,total,qr\ntask1,10,0x000|h160\ntask2,20,0x100|h160".parse().unwrap()
    }

    fn mock_file_details() -> Self {
        "inv_date,inv_no,from_attn_name,from_org_name,from_org_addr,from_email,to_attn_name,to_title,to_org_name,to_email\n01.04.2023,0001,luke,clawbird,1 metaverse ave,test@test.com,recipient_name,director,nftverse,test2@test.com\ninvoice date,invoice number,name person from,organisation name from,organisation address from,email from,name person attention to,title to,organisation name to,email to".parse().unwrap()
    }

    pub fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    pub fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}
