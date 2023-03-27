// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use gloo_storage::{LocalStorage, Storage};

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

pub struct State {
    pub editing: Editing,
    pub name: String,
    pub table: Table,
    pub entry: Entry,
    pub entry_editing: bool,
}

pub struct Entry {
    pub description: String,
    pub entry_editing: bool,
}

pub struct Table {
    pub source: TextSource,
    pub columns: Vec<Text>,
    pub rows: Vec<Vec<Text>>,
}

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
            entry_editing: false,
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
        let entry_editing = vec[1].to_string().parse::<bool>().or_else(|_i| Err(Error::ParseBoolError));
        let _entry_editing = match entry_editing {
            Ok(entry_editing) => {
                Ok(Entry { description, entry_editing })
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
            name:  "<no file>".to_owned(),
            table: Table::mock(),
            entry: Entry {
                description: description.to_owned(),
                entry_editing: false,
            },
            entry_editing: false,
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing: Editing::None,
            name: "<no file>".to_owned(),
            table: Table::mock(),
            entry: Entry {
                description: "<no entry>".to_owned(),
                entry_editing: false,
            },
            entry_editing: false,
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
        self.entry_editing = true;

        self.store();
    }

    pub fn add(&mut self, description: String) {
        self.entry = Entry {
            description,
            entry_editing: false,
        };

        self.store();
    }

    pub fn update(&mut self, description: String) {
        let entry = &mut self.entry;
        let entry_editing = &mut self.entry_editing;

        *entry_editing = false;

        if description != entry.description {
            entry.description = description;
            self.store();
        }
    }
}

impl Deref for State {
    type Target = Table;

    fn deref(&self) -> &Table {
        &self.table
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

pub struct TextSource {
    source: String,
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

    pub fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    pub fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}
