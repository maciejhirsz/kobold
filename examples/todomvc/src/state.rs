use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::UnwrapThrowExt;

use crate::filter::Filter;

const KEY: &str = "kobold.todomvc.example";

pub struct State {
    pub entries: Vec<Entry>,
    pub filter: Filter,
    pub editing: Option<usize>,
}

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

    fn read(from: &str) -> Option<Self> {
        let description = from.get(1..).map(Into::into)?;
        let completed = from.starts_with('+');

        Some(Entry {
            description,
            completed,
            editing: false,
        })
    }

    fn write(&self, storage: &mut String) {
        storage.extend([
            if self.completed { "+" } else { "-" },
            &self.description,
            "\n",
        ]);
    }
}

impl Default for State {
    fn default() -> Self {
        let mut entries = Vec::new();

        if let Some(storage) = LocalStorage::raw().get_item(KEY).ok().flatten() {
            entries.extend(storage.lines().map_while(Entry::read));
        }

        let hash = web_sys::window()
            .expect_throw("no window")
            .location()
            .hash();

        let filter = match hash.as_ref().map(|s| s.as_str()).unwrap_or("") {
            "#/active" => Filter::Active,
            "#/completed" => Filter::Completed,
            _ => Filter::All,
        };

        State {
            entries,
            filter,
            editing: None,
        }
    }
}

impl State {
    #[inline(never)]
    pub fn store(&self) {
        let capacity = self
            .entries
            .iter()
            .map(|entry| entry.description.len() + 3)
            .sum();

        let mut storage = String::with_capacity(capacity);

        for entry in &self.entries {
            entry.write(&mut storage);
        }

        LocalStorage::raw().set_item(KEY, &storage).ok();
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

    pub fn clear(&mut self) {
        self.entries.retain(|entry| !entry.completed);

        self.store();
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
