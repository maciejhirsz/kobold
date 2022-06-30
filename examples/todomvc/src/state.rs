use crate::filter::Filter;

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
}

impl State {
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
    }

    pub fn add(&mut self, description: String) {
        self.entries.push(Entry {
            description,
            completed: false,
            editing: false,
        });
    }

    pub fn remove(&mut self, idx: usize) {
        self.entries.remove(idx);
    }

    pub fn update(&mut self, idx: usize, description: String) {
        let entry = &mut self.entries[idx];

        entry.description = description;
        entry.editing = false;
    }

    pub fn toggle(&mut self, idx: usize) {
        self.entries[idx].completed ^= true;
    }
}
