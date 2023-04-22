use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node` or length if it's a tail node
    meta: Meta<T>,

    /// All the elements of the `Node` in
    data: [MaybeUninit<T>],
}

enum Meta<T> {
    Next(NonNull<Node<T>>),
    Tail(usize),
}

union FatPtr<T> {
    raw: (NonNull<Meta<T>>, usize),
    fat: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of elements in the list
    len: usize,

    /// First `Node` in the list
    first: NonNull<Node<T>>,
}

impl<T> Node<T> {
    const MIN_PAGE_SIZE: usize = {
        let n = 1024 / std::mem::size_of::<T>();

        if n == 0 {
            1
        } else {
            n
        }
    };

    fn new(cap: usize) -> NonNull<Self> {
        let cap = std::cmp::max(cap, Self::MIN_PAGE_SIZE);

        debug_assert_eq!(
            std::mem::size_of::<(NonNull<Meta<T>>, usize)>(),
            std::mem::size_of::<NonNull<Node<T>>>()
        );

        Vec::<u32>::new().into_boxed_slice();

        unsafe {
            let meta = NonNull::new_unchecked(alloc(Self::layout(cap)) as *mut Meta<T>);

            meta.as_ptr().write(Meta::Tail(0));

            FatPtr { raw: (meta, cap) }.fat
        }
    }

    fn dangling() -> NonNull<Self> {
        unsafe {
            let head = NonNull::dangling();

            FatPtr { raw: (head, 0) }.fat
        }
    }

    fn dealloc(ptr: NonNull<Self>) {
        unsafe { dealloc(ptr.as_ptr().cast(), Layout::for_value(ptr.as_ref())) }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn len(&self) -> usize {
        match self.meta {
            Meta::Next(_) => self.capacity(),
            Meta::Tail(len) => len,
        }
    }

    // unsafe fn assume_page(&mut self) -> &mut [T] {
    //     &mut *(&mut self.data as *mut _ as *mut [T])
    // }

    unsafe fn assume_slice(&mut self, len: usize) -> &mut [T] {
        &mut *(&mut self.data[..len] as *mut _ as *mut [T])
    }

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }

    const fn layout(cap: usize) -> Layout {
        use std::mem::{align_of, size_of};

        let mut align = align_of::<Meta<T>>();
        let mut pad = 0;

        if align_of::<T>() > align {
            pad = align_of::<T>() - align;
            align = align_of::<T>();
        }

        unsafe {
            Layout::from_size_align_unchecked(
                size_of::<Meta<T>>() + pad + cap * size_of::<T>(),
                align,
            )
        }
    }
}

impl<T> LinkedList<T> {
    pub fn build<I, U, F>(iter: I, mut constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut first = Meta::Tail(0); //Node::dangling();

        let mut iter = iter.into_iter();
        let mut next = &mut first;
        let mut len = 0;

        unsafe {
            while let Some(item) = iter.next() {
                let node = Node::new(iter.size_hint().0 + 1);
                *next = Meta::Next(node);
                let node = Node::as_mut(node);

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                let mut node_len = 1;

                for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

                    node_len += 1;
                }

                len += node_len;

                node.meta = Meta::Tail(node_len);

                next = &mut node.meta;
            }
        }

        let first = match first {
            Meta::Next(node) => node,
            Meta::Tail(_) => Node::dangling(),
        };

        LinkedList { len, first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            cur: &mut self.first,
            len: &mut self.len,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut node;

        unsafe {
            while self.len > 0 {
                node = Node::as_mut(self.first);

                let len = match node.meta {
                    Meta::Next(next) => {
                        self.first = next;
                        Node::as_mut(next).capacity()
                    }
                    Meta::Tail(len) => len,
                };

                drop_in_place(node.assume_slice(len));

                self.len -= len;

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    cur: &'cur mut NonNull<Node<T>>,
    len: &'cur mut usize,
}

impl<'cur, T> Iterator for Cursor<'cur, T>
where
    T: 'cur,
{
    type Item = &'cur mut T;

    fn next(&mut self) -> Option<&'cur mut T> {
        let cur = if self.idx == *self.len {
            return None;
        } else {
            Node::as_mut(*self.cur)
        };

        let cap = cur.data.len();
        let item = unsafe { cur.data[self.idx].assume_init_mut() };

        self.idx += 1;

        if self.idx == cap {
            if let Meta::Next(next) = &mut cur.meta {
                self.idx = 0;
                self.cur = next;
            }
        }

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list() {
        let list = LinkedList::build([], |n: usize, p| p.put(n));

        assert_eq!(list.len, 0);
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..128, |n, p| p.put(n));

        assert_eq!(list.len, 128);

        let first = Node::as_mut(list.first);

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[127].assume_init_read(), 127);
        }
    }

    #[test]
    fn one_node_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.first);

            assert_eq!(**first.data[0].assume_init_ref(), 0);
            assert_eq!(**first.data[19].assume_init_ref(), 19);
        }
    }

    #[test]
    fn cursor_iter() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(n));

        assert_eq!(list.len, 100);

        for (left, right) in list.cursor().zip(0..100) {
            assert_eq!(*left, right);
        }
    }
}
