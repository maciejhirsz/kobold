use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

use crate::filter::Filter;

const KEY: &str = "kobold.todomvc.example";

pub struct State {
    pub entries: Vec<Entry>,
    pub filter: Filter,
    pub editing: Option<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub description: String,
    pub completed: bool,
    pub editing: bool,
}

impl Entry {
    fn filter(&self, f: Filter) -> bool {
        match f {
            Filter::All => true,
            Filter::Active => !self.completed,
            Filter::Completed => self.completed,
        }
    }
}

impl State {
    pub fn load() -> Self {
        let entries = LocalStorage::get(KEY).unwrap_or_default();

        State {
            entries,
            filter: Filter::All,
            editing: None,
        }
    }

    #[inline(never)]
    pub fn store(&self) {
        LocalStorage::set(KEY, &self.entries).expect("failed to set");
    }

    pub fn count_active(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.filter(Filter::Active))
            .count()
    }

    pub fn filtered_entries(&self) -> impl Iterator<Item = (usize, &Entry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.filter(self.filter))
    }

    pub fn set_all(&mut self, completed: bool) {
        for entry in self.entries.iter_mut() {
            entry.completed = completed;
        }
    }

    pub fn edit_entry(&mut self, idx: usize) {
        if let Some(entry) = self.editing.and_then(|idx| self.entries.get_mut(idx)) {
            entry.editing = false;
        }

        self.editing = Some(idx);
        self.entries[idx].editing = true;

        self.store();
    }

    pub fn add(&mut self, description: String) {
        self.entries.push(Entry {
            description,
            completed: false,
            editing: false,
        });

        self.store();
    }

    pub fn remove(&mut self, idx: usize) {
        self.entries.remove(idx);

        self.store();
    }

    pub fn update(&mut self, idx: usize, description: String) {
        let entry = &mut self.entries[idx];

        entry.editing = false;

        if description != entry.description {
            entry.description = description;
            self.store();
        }
    }

    pub fn toggle(&mut self, idx: usize) {
        self.entries[idx].completed ^= true;

        self.store();
    }
}
