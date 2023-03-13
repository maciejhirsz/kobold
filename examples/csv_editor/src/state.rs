use std::ops::{Deref, DerefMut, Range};

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

impl Default for Text {
    fn default() -> Self {
        Text::Insitu(0..0)
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing: Editing::None,
            name: "<no file>".to_owned(),
            table: Table::mock(),
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
