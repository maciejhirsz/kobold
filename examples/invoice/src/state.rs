// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::UnwrapThrowExt;
// use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};

use std::ops::{Deref, DerefMut, Range};

const KEY: &str = "kobold.invoice.example";

#[derive(Debug)]
pub enum Error {
    FailedToParseEntry,
    ParseBoolError,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

pub struct Content {
    pub name: String,
    pub table: Table,
}

pub struct State {
    pub editing: Editing,
    pub main: Content,
    pub details: Content,
    pub entry: Entry,
    pub qr_code: String,
}

pub struct Entry {
    pub description: String,
    pub editing: bool,
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

impl Entry {
    fn mock() -> Self {
        "my address\nyes".parse().unwrap()
    }

    fn read(from: &str) -> Option<Self> {
        let description = from.to_string();

        Some(Entry {
            description,
            editing: false,
        })
    }

    fn write(&self, storage: &mut String) {
        storage.extend([
            &self.description,
            "\n",
        ]);
    }
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        let vec = input.lines().collect::<Vec<_>>();
        let description = vec[0].to_string();
        let editing = vec[1].to_string().parse::<bool>().or_else(|_i| Err(Error::ParseBoolError));
        let _editing = match editing {
            Ok(editing) => {
                Ok(Entry { description, editing })
            },
            Err(_) => {
                Err(Error::FailedToParseEntry)
            }
        };
        Err(Error::FailedToParseEntry)
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::Insitu(0..0)
    }
}

impl Default for State {
    fn default() -> Self {
        let mut description = String::new();
        if let Some(storage) = LocalStorage::raw().get(KEY).ok() {
            description = storage.unwrap();
        }

        State {
            editing: Editing::None,
            main: Content {
                name: "<no main file>".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "<no details file>".to_owned(),
                table: Table::mock_file_details(),   
            },
            entry: Entry {
                description: description.to_owned(),
                editing: false,
            },
            qr_code: "0x000".to_string(),
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing: Editing::None,
            main: Content {
                name: "<no main file>".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "<no details file>".to_owned(),
                table: Table::mock_file_details(),   
            },
            entry: Entry {
                description: "<enter billing address>".to_owned(),
                editing: false,
            },
            qr_code: "0x000".to_string(),
        }
    }

    #[inline(never)]
    pub fn store(&self) {
        let capacity = self.entry.description.len() + 3;

        let mut storage = String::with_capacity(capacity);

        self.entry.write(&mut storage);

        LocalStorage::raw().set_item(KEY, &storage).ok();
    }

    pub fn edit_entry(&mut self) {
        self.entry.editing = true;

        self.store();
    }

    pub fn add(&mut self, description: String) {
        self.entry = Entry {
            description,
            editing: false,
        };

        self.store();
    }

    pub fn update(&mut self, description: String) {
        let entry = &mut self.entry;
        entry.editing = false;

        if description != entry.description {
            entry.description = description;
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
        "column 1,column 2\nA1,A2\nB1,B2".parse().unwrap()
    }

    fn mock_file_details() -> Self {
        "inv_date,inv_no,from_attn_name,from_org_name,from_org_addr,from_email,to_attn_name,to_title,to_org_name,to_email\n
01.04.2023,0001,luke,clawbird,1 metaverse ave,test@test.com,recipient_name,director,nftverse,test2@test.com\n
invoice date,invoice number,name person from,organisation name from,organisation address from,email from,name person attention to,title to,organisation name to,email to".parse().unwrap()
    }

    pub fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    pub fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}
