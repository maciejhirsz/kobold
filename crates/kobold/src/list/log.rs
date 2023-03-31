use std::cell::Cell;
use std::cmp::max;
use std::ops::{Deref, Index, IndexMut, Range, RangeFull};
use std::ops::{RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::vec::Drain;

enum Entry {
    Update(Range<usize>),
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
            log: ChangeLog {
                log: Cell::new(Vec::new()),
            },
        }
    }
}

impl ChangeLog {
    fn update(&mut self, upd: Range<usize>) {
        if let Some(Entry::Update(previous)) = self.log.get_mut().last_mut() {
            if let Some(new) = join(previous.clone(), upd.clone()) {
                *previous = new;
                return;
            }
        }
        self.log.get_mut().push(Entry::Update(upd));
    }

    fn update_one(&mut self, index: usize) {
        self.update(index..index + 1);
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

    fn push(&mut self, index: usize) {
        match self.log.get_mut().last_mut() {
            Some(Entry::Insert(previous)) if previous.end == index => {
                previous.end += 1;
            }
            _ => self.log.get_mut().push(Entry::Insert(index..index + 1)),
        }
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
    pub fn drain(&mut self, range: Range<usize>) -> Drain<T> {
        self.log.remove(range.clone());
        self.data.drain(range)
    }

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
            self.log.update_one(index);
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
        self.log.push(self.data.len());
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
        let reverse_from = self.log.log.get_mut().len();

        let mut index = 0;
        self.data.retain_mut(|elem| {
            let retain = f(elem);

            if !retain {
                self.log.remove_one(index);
            }

            index += 1;
            retain
        });

        // Reverse the ranges so they can be drained in order
        // without issues.
        self.log.log.get_mut()[reverse_from..].reverse()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }

        self.data.swap(a, b);
        self.log.update_one(a);
        self.log.update_one(b);
    }

    pub fn touch(&mut self, range: impl AsRange) {
        self.log.update(range.as_range(self.data.len()));
    }
}

pub trait AsRange {
    fn as_range(&self, len: usize) -> Range<usize>;
}

macro_rules! as_range {
    ($($r:ty [$self:ident, $len:tt, $code:expr],)*) => {
        $(
            impl AsRange for $r {
                fn as_range(&$self, $len: usize) -> Range<usize> {
                    $code
                }
            }
        )*
    };
}

as_range! {
    usize [self, _, *self..*self + 1],
    Range<usize> [self, _, self.clone()],
    RangeInclusive<usize> [self, _, *self.start()..*self.end() + 1],
    RangeFull [self, len, 0..len],
    RangeFrom<usize> [self, len, self.start..len],
    RangeTo<usize> [self, _, 0..self.end],
    RangeToInclusive<usize> [self, _, 0..self.end + 1],
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

impl<T, I> Index<I> for Log<Vec<T>>
where
    Vec<T>: Index<I>,
{
    type Output = <Vec<T> as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T, I> IndexMut<I> for Log<Vec<T>>
where
    I: AsRange,
    Vec<T>: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.log.update(index.as_range(self.data.len()));
        self.data.index_mut(index)
    }
}

fn join(mut a: Range<usize>, mut b: Range<usize>) -> Option<Range<usize>> {
    if a.contains(&b.start) {
        a.end = max(a.end, b.end);
        Some(a)
    } else if b.contains(&a.start) {
        b.end = max(b.end, a.end);
        Some(b)
    } else {
        None
    }
}
