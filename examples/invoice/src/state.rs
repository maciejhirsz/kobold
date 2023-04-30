// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::{debug, error, info, warn};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::ops::{Deref, DerefMut, Range};
use wasm_bindgen::UnwrapThrowExt;

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
    pub table: Table,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub editing_main: Editing,
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
        let main_local_storage: Table = match LocalStorage::get(KEY_MAIN) {
            Ok(local_storage) => local_storage,
            Err(err) => Table::mock(),
        };
        let details_local_storage: Table = match LocalStorage::get(KEY_DETAILS) {
            Ok(local_storage) => local_storage,
            // TODO - check that this actually converts to Table type
            Err(err) => Table::mock_file_details(),
        };
        debug!(
            "loading local storage: {:?}\n\n{:?}",
            main_local_storage, details_local_storage
        );

        State {
            editing_main: Editing::None,
            editing_details: Editing::None,
            main: Content {
                filename: "main.csv".to_owned(),
                table: main_local_storage,
            },
            details: Content {
                filename: "details.csv".to_owned(),
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
                table: Table::mock(),
            },
            details: Content {
                filename: "details.csv".to_owned(),
                table: Table::mock_file_details(),
            },
        }
    }

    #[inline(never)]
    // store the updated state in web browser local storage
    pub fn store(&self) {
        debug!(
            "updating store: {:?}\n\n{:?}",
            &self.main.table, &self.details.table
        );
        LocalStorage::set(KEY_MAIN, &self.main.table).unwrap_throw();
        LocalStorage::set(KEY_DETAILS, &self.details.table).unwrap_throw();
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
        "description,total,qr\ntask1,10,0x000|h160\ntask2,20,0x100|h160"
            .parse()
            .unwrap_throw()
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
