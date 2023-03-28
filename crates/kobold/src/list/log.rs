use std::ops::{Deref, Index, IndexMut, Range};
use std::cell::Cell;

enum Entry {
    Mutate(usize),
    Insert(Range<usize>),
    Remove(Range<usize>),
}

pub struct Log<T> {
    data: T,
    log: ChangeLog,
}

impl<T> Deref for Log<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct ChangeLog {
    log: Cell<Vec<Entry>>,
}

impl<T> Log<T> {
    pub const fn new(data: T) -> Self {
        Log {
            data,
            log: ChangeLog { log: Cell::new(Vec::new()) },
        }
    }
}

impl ChangeLog {
    fn mutate(&mut self, index: usize) {
        match self.log.get_mut().last_mut() {
            Some(Entry::Mutate(previous)) if *previous == index => (),
            _ => self.log.get_mut().push(Entry::Mutate(index)),
        }
    }

    fn insert(&mut self, ins: Range<usize>) {
        match self.log.get_mut().last_mut() {
            Some(Entry::Insert(previous)) if previous.end == ins.start => {
                previous.end = ins.start;
            }
            Some(Entry::Insert(previous)) if previous.start == ins.start => {
                previous.end += ins.end - ins.start;
            }
            _ => self.log.get_mut().push(Entry::Insert(ins)),
        }
    }

    fn insert_one(&mut self, index: usize) {
        self.insert(index..index + 1);
    }

    fn remove(&mut self, rem: Range<usize>) {
        match self.log.get_mut().last_mut() {
            Some(Entry::Remove(previous)) if previous.end == rem.start => {
                previous.end = rem.end;
            }
            Some(Entry::Remove(previous)) if previous.start == rem.start => {
                previous.end += rem.end - rem.start;
            }
            _ => self.log.get_mut().push(Entry::Remove(rem)),
        }
    }

    fn remove_one(&mut self, index: usize) {
        self.remove(index..index + 1)
    }
}

impl<T> Log<Vec<T>> {
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.log
            .insert(self.data.len()..self.data.len() + other.len());
        self.data.extend_from_slice(other);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let item = self.data.get_mut(index);

        if item.is_some() {
            self.log.mutate(index);
        }

        item
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.log.insert_one(index);
        self.data.insert(index, element)
    }

    pub fn pop(&mut self) -> Option<T> {
        let pop = self.data.pop();

        if pop.is_some() {
            self.log.remove_one(self.data.len());
        }

        pop
    }

    pub fn push(&mut self, val: T) {
        self.log.insert_one(self.data.len());
        self.data.push(val);
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.log.remove(index..index + 1);
        self.data.remove(index)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|elem| f(elem));
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut index = 0;
        self.data.retain_mut(|elem| {
            let retain = f(elem);

            if !retain {
                self.log.remove_one(index);
            }

            index += 1;
            retain
        });
    }
}

impl<T, E> Extend<E> for Log<Vec<T>>
where
    Vec<T>: Extend<E>,
{
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        let start = self.data.len();
        self.data.extend(iter);
        self.log.insert(start..self.data.len());
    }
}

impl<T> Index<usize> for Log<Vec<T>> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self[index]
    }
}

impl<T> IndexMut<usize> for Log<Vec<T>> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.log.mutate(index);
        &mut self[index]
    }
}
