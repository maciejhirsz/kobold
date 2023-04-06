// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::UnwrapThrowExt;
// use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
use log::{info, debug, error, warn};

use std::ops::{Deref, DerefMut, Range};

const KEY_MAIN: &str = "kobold.invoice.main";
const KEY_DETAILS: &str = "kobold.invoice.details";

// #[derive(Debug)]
// pub enum Error {
//     FailedToParseEntry,
//     ParseBoolError,
// }

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
    pub editing_details: Editing,
    pub main: Content,
    pub details: Content,
    // pub entry: Vec<Entry>,
}

// pub struct Entry {
//     pub description: String,
//     pub editing: bool,
// }

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

// impl Entry {
//     fn mock() -> Self {
//         "my address\nyes".parse().unwrap()
//     }

//     fn read(from: &str) -> Option<Self> {
//         let description = from.to_string();

//         Some(Entry {
//             description,
//             editing: false,
//         })
//     }

//     fn write(&self, storage: &mut String) {
//         storage.extend([
//             &self.description,
//             "\n",
//         ]);
//     }
// }

// impl FromStr for Entry {
//     type Err = Error;

//     fn from_str(input: &str) -> Result<Self, Error> {
//         let vec = input.lines().collect::<Vec<_>>();
//         let description = vec[0].to_string();
//         let editing = vec[1].to_string().parse::<bool>().or_else(|_i| Err(Error::ParseBoolError));
//         let _editing = match editing {
//             Ok(editing) => {
//                 Ok(Entry { description, editing })
//             },
//             Err(_) => {
//                 Err(Error::FailedToParseEntry)
//             }
//         };
//         Err(Error::FailedToParseEntry)
//     }
// }

impl Default for Text {
    fn default() -> Self {
        Text::Insitu(0..0)
    }
}

impl Default for State {
    fn default() -> Self {
        // let (mut index, mut description) = (0, String::with_capacity(3));
        // let mut storage = format!("{:#?}\n{:#?}", index, description);
        let mut default_data = "_,_,_,_";
        let mut storage = format!("{:#?},{:#?},{:#?},{:#?}", default_data);

        LocalStorage::raw().set_item(KEY_MAIN, &storage).ok();
        LocalStorage::raw().set_item(KEY_DETAILS, &storage).ok();

        // if let Some(_storage) = LocalStorage::raw().get(KEY_MAIN).ok() {
        //     storage = _storage.unwrap();
        // }


        State {
            editing: Editing::None,
            editing_details: Editing::None,
            main: Content {
                name: "<no main file>".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "<no details file>".to_owned(),
                table: Table::mock_file_details(),   
            },
            // entry: vec![Entry {
            //     description: description.to_owned(),
            //     editing: false,
            // }],
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing: Editing::None,
            editing_details: Editing::None,
            main: Content {
                name: "<no main file>".to_owned(),
                table: Table::mock(),
            },
            details: Content {
                name: "<no details file>".to_owned(),
                table: Table::mock_file_details(),   
            },
            // entry: vec![
            //     Entry {
            //         description: "<enter ??? address>".to_owned(),
            //         editing: false,
            //     }
            // ],
        }
    }

    #[inline(never)]
    pub fn store(&self, content_name: String, new_storage: String) {
        // let capacity = self.entry[index].description.len() + 3;
        // let (mut index, mut description) = (index, String::with_capacity(capacity));

        String::with_capacity(capacity));

        // self.entry[index].write(&mut storage);

        

        if content_name == KEY_MAIN.to_string() {
            LocalStorage::raw().set_item(KEY_MAIN, &new_storage).ok();
        } else if content_name == KEY_DETAILS.to_string() {
            LocalStorage::raw().set_item(KEY_DETAILS, &new_storage).ok();
        } else {
            throw_str("unknown content_name");
        }
    }

    // pub fn edit_entry(&mut self, index: usize) {
    //     self.entry[index].editing = true;

    //     self.store(index);
    // }

    pub fn edit(&mut self, index: usize) {
        // self.entry[index].editing = true;

        self.store(index);
    }

    // pub fn add(&mut self, index: usize, description: String) {
    //     self.entry[index] = Entry {
    //         description,
    //         editing: false,
    //     };

    //     self.store(index);
    // }

    pub fn add(&mut self, index: usize) {
        // self.entry[index] = Entry {
        //     description,
        //     editing: false,
        // };

        self.store(index);
    }

    // pub fn update(&mut self, index: usize, description: String) {
    //     let entry = &mut self.entry[index];
    //     entry.editing = false;

    //     if description != entry.description {
    //         entry.description = description;
    //         self.store(index);
    //     }
    // }

    pub fn update(&mut self, content_name: String, row: Text, col: Text, value: String) {

        // WRONG -- see todomvc example of fn store for use of LocalStorage

        // let new_storage = format!("{:#?},{:#?},{:#?}", row, col, value);
        // let mut existing_storage;
        // if content_name == KEY_MAIN.to_string() {
        //     if let Some(_storage) = LocalStorage::raw().get(KEY_MAIN).ok() {
        //         existing_storage = _storage.unwrap();
        //         debug!("existing_storage main: {:#?}", existing_storage);
        //     }
        // } else if content_name == KEY_DETAILS.to_string() {
        //     if let Some(_storage) = LocalStorage::raw().get(KEY_DETAILS).ok() {
        //         existing_storage = _storage.unwrap();
        //         debug!("existing_storage details: {:#?}", existing_storage);
        //     }
        // } else {
        //     throw_str("unknown content_name");
        // }

        // if new_storage != existing_storage {
        //     self.store(content_name, new_storage);
        // }
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
