// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::UnwrapThrowExt;
// use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
use serde_json::to_string;
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

fn convert_vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
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
    pub fn store(&self) {
        let main_str = to_string(&self.main.table).unwrap();
        let details_str = to_string(&self.details.table).unwrap();

        debug!("updating store: {:?}\n\n{:?}", main_str.as_str(), details_str.as_str());

        LocalStorage::raw().set_item(KEY_MAIN, main_str.as_str()).ok();
        LocalStorage::raw().set_item(KEY_DETAILS, details_str.as_str()).ok();
    }

    pub fn update_main(&mut self, row: usize, col: usize, value: String) {
        let old_storage = self.main.table.source.get_text(&self.main.table.rows[row][col]);
        if value != old_storage {
            self.store();
        }
    }

    pub fn update_details(&mut self, row: usize, col: usize, value: String) {
        let old_storage = self.details.table.source.get_text(&self.details.table.rows[row][col]);
        if value != old_storage {
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
