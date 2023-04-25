use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::ops::Range;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node`.
    next: Option<NonNull<Node<T>>>,

    /// Number of initialized elements in `data`.
    len: usize,

    /// All the elements of the `Node`
    data: [MaybeUninit<T>],
}

/// This struct needs to be the same shape as `Node<T>`, sans the unsized `data`
#[repr(C)]
struct Head<T> {
    /// Pointer to the next `Node`.
    next: Option<NonNull<Node<T>>>,

    /// Number of initialized elements in `data`.
    len: usize,
}

union FatPtr<T> {
    raw: (NonNull<Head<T>>, usize),
    fat: NonNull<Node<T>>,
}

pub struct PageList<T> {
    /// First `Node` in the list
    first: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    const MIN_PAGE_SIZE: usize = {
        if std::mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            let n = 512 / std::mem::size_of::<T>();

            if n == 0 {
                1
            } else {
                n
            }
        }
    };

    fn new(cap: usize) -> NonNull<Self> {
        let cap = std::cmp::max(cap, Self::MIN_PAGE_SIZE);

        debug_assert_eq!(
            std::mem::size_of::<(NonNull<Head<T>>, usize)>(),
            std::mem::size_of::<NonNull<Node<T>>>()
        );

        Vec::<u32>::new().into_boxed_slice();

        unsafe {
            let head = alloc(Self::layout(cap)) as *mut Head<T>;

            head.write(Head { next: None, len: 0 });

            FatPtr {
                raw: (NonNull::new_unchecked(head), cap),
            }
            .fat
        }
    }

    fn dealloc(ptr: NonNull<Self>) {
        unsafe { dealloc(ptr.as_ptr().cast(), Layout::for_value(ptr.as_ref())) }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    unsafe fn assume_slice(&mut self, range: Range<usize>) -> &mut [T] {
        &mut *(self.data.get_unchecked_mut(range) as *mut _ as *mut [T])
    }

    fn slice(&mut self) -> &mut [T] {
        unsafe { &mut *(self.data.get_unchecked_mut(..self.len) as *mut _ as *mut [T]) }
    }

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }

    const fn layout(cap: usize) -> Layout {
        use std::mem::{align_of, size_of};

        let mut align = align_of::<Head<T>>();
        let mut pad = 0;

        if align_of::<T>() > align {
            pad = align_of::<T>() - align;
            align = align_of::<T>();
        }

        unsafe {
            Layout::from_size_align_unchecked(
                size_of::<Head<T>>() + pad + cap * size_of::<T>(),
                align,
            )
        }
    }
}

impl<T> PageList<T> {
    pub fn build<I, U, F>(iter: I, constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut first = None;

        Tail { cur: &mut first }.extend(iter, constructor);

        PageList { first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            cur: &mut self.first,
        }
    }
}

impl<T> Drop for PageList<T> {
    fn drop(&mut self) {
        unsafe {
            while let Some(node) = self.first {
                let node = Node::as_mut(node);

                drop_in_place(node.slice());

                self.first = node.next;

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    cur: &'cur mut Option<NonNull<Node<T>>>,
}

pub struct Tail<'cur, T> {
    cur: &'cur mut Option<NonNull<Node<T>>>,
}

impl<'cur, T> Tail<'cur, T> {
    pub fn extend<I, U, F>(self, iter: I, mut constructor: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut iter = iter.into_iter();
        let mut next = self.cur;

        while let Some(item) = iter.next() {
            let node = Node::as_mut(*next.get_or_insert_with(|| Node::new(iter.size_hint().0 + 1)));

            In::pinned(
                unsafe { Pin::new_unchecked(node.data.get_unchecked_mut(node.len)) },
                |p| constructor(item, p),
            );

            node.len += 1;

            if node.len == node.capacity() {
                next = &mut node.next;
            }
        }
    }
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    pub fn truncate_rest(self) -> Tail<'cur, T> {
        let node = match self.cur {
            Some(node) => Node::as_mut(*node),
            None => return Tail { cur: self.cur },
        };

        drop(PageList {
            first: node.next.take(),
        });

        let len = node.len;

        unsafe {
            drop_in_place(node.assume_slice(self.idx..len));
        }

        node.len = self.idx;

        Tail { cur: self.cur }
    }

    pub fn zip_each<I, F, U>(&mut self, iter: I, mut each: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(&mut T, U),
    {
        let mut iter = iter.into_iter();

        while let Some(cur) = self.cur.map(Node::as_mut) {
            if let Some(slot) = cur.slice().get_mut(self.idx) {
                if let Some(item) = iter.next() {
                    each(slot, item);

                    self.idx += 1;

                    if self.idx == cur.len {
                        self.idx = 0;
                        self.cur = &mut cur.next;
                    }
                    continue;
                }
            }

            break;
        }
    }
}

#[cfg(test)]
impl<'cur, T> Iterator for Cursor<'cur, T>
where
    T: 'cur,
{
    type Item = &'cur mut T;

    fn next(&mut self) -> Option<&'cur mut T> {
        let cur = match self.cur.map(Node::as_mut) {
            Some(node) if self.idx < node.len => node,
            _ => return None,
        };

        let item = unsafe { cur.data.get_unchecked_mut(self.idx).assume_init_mut() };

        self.idx += 1;

        if self.idx == cur.len {
            self.idx = 0;
            self.cur = &mut cur.next;
        }

        Some(item)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Just a helper that disables size hints for an
    // iterator, forcing default allocation size
    struct NoHint<I>(I);

    impl<I> Iterator for NoHint<I>
    where
        I: Iterator,
    {
        type Item = I::Item;

        fn next(&mut self) -> Option<I::Item> {
            self.0.next()
        }
    }

    #[test]
    fn empty_list() {
        let list = PageList::build([], |n: usize, p| p.put(n));

        assert!(list.first.is_none());
    }

    #[test]
    fn one_node() {
        let list = PageList::build(0..128, |n, p| p.put(n));

        let first = Node::as_mut(list.first.unwrap());

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[127].assume_init_read(), 127);
        }
    }

    #[test]
    fn one_node_alloc() {
        let list = PageList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.first.unwrap());

            assert_eq!(**first.data[0].assume_init_ref(), 0);
            assert_eq!(**first.data[19].assume_init_ref(), 19);
        }
    }

    #[test]
    fn many_nodes() {
        let mut list = PageList::build(NoHint(0..256), |n, p| p.put(n));

        let first = Node::as_mut(list.first.unwrap());

        assert!(first.capacity() < 256);

        list.cursor().zip_each(0..256, |left, right| {
            assert_eq!(*left, right);
        });
    }

    #[test]
    fn cursor_iter() {
        let mut list = PageList::build(0..100, |n, p| p.put(n));

        list.cursor().zip_each(0..100, |left, right| {
            assert_eq!(*left, right);
        });
    }

    #[test]
    fn cursor_truncate_unaligned() {
        let mut list = PageList::build(NoHint(0..300), |n, p| p.put(Box::new(n)));

        let mut cur = list.cursor();

        cur.by_ref().take(100).count();
        cur.truncate_rest();

        list.cursor().zip_each(0..200, |left, right| {
            assert_eq!(**left, right);
        });
    }

    #[test]
    fn cursor_truncate_extend_unaligned() {
        let mut list = PageList::build(NoHint(0..300), |n, p| p.put(Box::new(n)));

        let mut cur = list.cursor();

        cur.by_ref().take(100).count();
        cur.truncate_rest()
            .extend(200..300, |n, p| p.put(Box::new(n)));

        list.cursor()
            .zip_each((0..100).chain(200..300), |left, right| {
                assert_eq!(**left, right);
            });
    }

    #[test]
    fn cursor_truncate_extend_empty() {
        let mut list = PageList::build(NoHint(0..300), |n, p| p.put(Box::new(n)));

        let mut cur = list.cursor();

        cur.by_ref().take(100).count();
        cur.truncate_rest().extend([], |n, p| p.put(Box::new(n)));

        list.cursor().zip_each(0..100, |left, right| {
            assert_eq!(**left, right);
        });
    }

    #[test]
    fn cursor_truncate_extend_aligned() {
        let mut list = PageList::build(NoHint(0..512), |n, p| p.put(Box::new(n)));

        let mut cur = list.cursor();

        cur.by_ref().take(256).count();
        cur.truncate_rest()
            .extend(512..640, |n, p| p.put(Box::new(n)));

        list.cursor()
            .zip_each((0..256).chain(512..640), |left, right| {
                assert_eq!(**left, right);
            });
    }
}
