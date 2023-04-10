use std::cell::Cell;
use std::cmp::max;
use std::fmt::{self, Debug};
use std::ops::{Deref, Index, IndexMut, Range, RangeBounds, RangeFull};
use std::ops::{RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::vec::Drain;

enum Entry {
    Update(Range<usize>),
    Insert(Range<usize>),
    Remove(Range<usize>),
}

pub struct Tracking<T> {
    data: Vec<T>,
    log: ChangeLog,
}

impl<T> Debug for Tracking<T>
where
    Vec<T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl<T, U> PartialEq<U> for Tracking<T>
where
    Vec<T>: PartialEq<U>,
{
    fn eq(&self, other: &U) -> bool {
        self.data.eq(other)
    }
}

impl<T> Deref for Tracking<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct ChangeLog {
    log: Cell<Vec<Entry>>,
}

impl<T> Tracking<T> {
    pub const fn new(data: Vec<T>) -> Self {
        Tracking {
            data,
            log: ChangeLog {
                log: Cell::new(Vec::new()),
            },
        }
    }

    pub fn touch(&mut self, range: impl AsRange) {
        self.log.update(range.as_range(self.data.len()));
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

/// `Vec<T>` proxied methods.
impl<T> Tracking<T> {
    /// Removes the specified range from the vector in bulk, returning all
    /// removed elements as an iterator. If the iterator is dropped before
    /// being fully consumed, it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the vector to optimize
    /// its implementation.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements arbitrarily, including elements outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut v = Tracking::new(vec![1, 2, 3]);
    /// let u: Vec<_> = v.drain(1..).collect();
    /// assert_eq!(v, &[1]);
    /// assert_eq!(u, &[2, 3]);
    ///
    /// // A full range clears the vector, like `clear()` does
    /// v.drain(..);
    /// assert_eq!(v, &[]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: RangeBounds<usize> + AsRange,
    {
        self.log.remove(range.as_range(self.data.len()));
        self.data.drain(range)
    }

    /// Clones and appends all elements in a slice to the `Vec`.
    ///
    /// Iterates over the slice `other`, clones each element, and then appends
    /// it to this `Vec`. The `other` slice is traversed in-order.
    ///
    /// Note that this function is same as [`extend`] except that it is
    /// specialized to work with slices instead. If and when Rust gets
    /// specialization this function will likely be deprecated (but still
    /// available).
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1]);
    /// vec.extend_from_slice(&[2, 3, 4]);
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`extend`]: Tracking::extend
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.log
            .insert(self.data.len()..self.data.len() + other.len());
        self.data.extend_from_slice(other);
    }

    /// Returns a mutable reference to an element or subslice depending on the
    /// type of index (see [`get`]) or `None` if the index is out of bounds.
    ///
    /// [`get`]: slice::get
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut x = Tracking::new(vec![0, 1, 2]);
    ///
    /// if let Some(elem) = x.get_mut(1) {
    ///     *elem = 42;
    /// }
    /// assert_eq!(x, &[0, 42, 2]);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let item = self.data.get_mut(index);

        if item.is_some() {
            self.log.update_one(index);
        }

        item
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1, 2, 3]);
    /// vec.insert(1, 4);
    /// assert_eq!(vec, [1, 4, 2, 3]);
    /// vec.insert(4, 5);
    /// assert_eq!(vec, [1, 4, 2, 3, 5]);
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        self.log.insert_one(index);
        self.data.insert(index, element)
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1, 2, 3]);
    /// assert_eq!(vec.pop(), Some(3));
    /// assert_eq!(vec, [1, 2]);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        let pop = self.data.pop();

        if pop.is_some() {
            self.log.remove_one(self.data.len());
        }

        pop
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1, 2]);
    /// vec.push(3);
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    pub fn push(&mut self, val: T) {
        self.log.push(self.data.len());
        self.data.push(val);
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// Note: Because this shifts over the remaining elements, it has a
    /// worst-case performance of *O*(*n*). If you don't need the order of elements
    /// to be preserved, use [`swap_remove`] instead.
    ///
    /// [`swap_remove`]: Tracking::swap_remove
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut v = Tracking::new(vec![1, 2, 3]);
    /// assert_eq!(v.remove(1), 2);
    /// assert_eq!(v, [1, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        self.log.remove(index..index + 1);
        self.data.remove(index)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = vec![1, 2, 3, 4];
    /// vec.retain(|&x| x % 2 == 0);
    /// assert_eq!(vec, [2, 4]);
    /// ```
    ///
    /// Because the elements are visited exactly once in the original order,
    /// external state may be used to decide which elements to keep.
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1, 2, 3, 4, 5]);
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// vec.retain(|_| *iter.next().unwrap());
    /// assert_eq!(vec, [2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|elem| f(elem));
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut vec = Tracking::new(vec![1, 2, 3, 4]);
    /// vec.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    /// assert_eq!(vec, [2, 3, 4]);
    /// ```
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

    /// Swaps two elements in the vector.
    ///
    /// # Arguments
    ///
    /// * a - The index of the first element
    /// * b - The index of the second element
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut v = Tracking::new(vec!["a", "b", "c", "d", "e"]);
    /// v.swap(2, 4);
    /// assert!(v == ["a", "b", "e", "d", "c"]);
    /// ```
    pub fn swap(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }

        self.data.swap(a, b);
        self.log.update_one(a);
        self.log.update_one(b);
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is *O*(1).
    /// If you need to preserve the element order, use [`remove`] instead.
    ///
    /// [`remove`]: Tracking::remove
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use kobold::list::Tracking;
    ///
    /// let mut v = Tracking::new(vec!["foo", "bar", "baz", "qux"]);
    ///
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(v, ["foo", "qux", "baz"]);
    ///
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(v, ["baz", "qux"]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        let last = self.data.len() - 1;
        if index < last {
            self.log.update_one(index);
        }
        self.log.remove_one(last);
        self.data.swap_remove(index)
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

impl<T, E> Extend<E> for Tracking<T>
where
    Vec<T>: Extend<E>,
{
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        let start = self.data.len();
        self.data.extend(iter);
        self.log.insert(start..self.data.len());
    }
}

impl<T, I> Index<I> for Tracking<T>
where
    Vec<T>: Index<I>,
{
    type Output = <Vec<T> as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T, I> IndexMut<I> for Tracking<T>
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
