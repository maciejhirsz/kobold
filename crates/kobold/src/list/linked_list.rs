use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

const PAGE_SIZE: usize = 16;

struct Node<T> {
    /// Array to store all the elements of the `Node` in
    data: [MaybeUninit<T>; PAGE_SIZE],

    len: usize,

    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    next: Option<NonNull<Node<T>>>,
}

pub struct LinkedList<T> {
    /// First `Node` in the list
    first: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    fn new() -> NonNull<Self> {
        use std::alloc::{alloc, Layout};
        use std::ptr::addr_of_mut;

        unsafe {
            let ptr = alloc(Layout::new::<Self>()) as *mut Self;

            addr_of_mut!((*ptr).len).write(0);
            addr_of_mut!((*ptr).next).write(None);

            NonNull::new_unchecked(ptr) }
    }

    unsafe fn dealloc(ptr: NonNull<Self>) {
        use std::alloc::{dealloc, Layout};

        dealloc(ptr.as_ptr().cast(), Layout::new::<Self>());
    }

    unsafe fn assume_slice(&mut self) -> &mut [T] {
        &mut *(self.data.get_unchecked_mut(..self.len) as *mut _ as *mut [T])
    }

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }
}

impl<T> LinkedList<T> {
    pub fn build<I, U, F>(iter: I, mut constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut first = None;

        let mut iter = iter.into_iter();
        let mut next = &mut first;

        unsafe {
            while let Some(item) = iter.next() {
                *next = Some(Node::new());

                let node = Node::as_mut(next.unwrap());

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                node.len = 1;

                for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

                    node.len += 1;
                }

                next = &mut node.next;
            }
        }

        LinkedList { first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            cur: &mut self.first,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        unsafe {
            while let Some(node) = self.first {
                let node = Node::as_mut(node);

                drop_in_place(node.assume_slice());

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

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    pub fn truncate_rest(self) -> Tail<'cur, T> {
        let first = if self.idx == 0 {
            self.cur.take()
        } else if let Some(node) = *self.cur {
            let node = Node::as_mut(node);

            unsafe {
                drop_in_place(&mut node.assume_slice()[self.idx..]);
            }

            node.len = self.idx;
            node.next.take()
        } else {
            None
        };

        drop(LinkedList {
            first,
        });

        Tail {
            cur: self.cur,
        }
    }

    pub fn has_next(&self) -> bool {
        if let Some(node) = *self.cur {
            self.idx < Node::as_mut(node).len
        } else {
            false
        }
    }

    pub fn pair<I, F, U>(&mut self, iter: I, mut each: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(&mut T, U),
    {
        let mut iter = iter.into_iter();

        while self.has_next() {
            if let Some(item) = iter.next() {
                each(self.next().unwrap(), item);
            } else {
                break;
            }
        }
    }
}

impl<'cur, T> Tail<'cur, T> {
    pub fn extend<I, U, F>(self, iter: I, mut constructor: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut iter = iter.into_iter();
        let mut next = self.cur;

        unsafe {
            if let Some(node) = *next {
                let node = Node::as_mut(node);

                for (slot, item) in node.data[node.len..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));
                    node.len += 1;
                }

                next = &mut node.next;
            }

            while let Some(item) = iter.next() {
                *next = Some(Node::new());

                let node = Node::as_mut(next.unwrap());

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                node.len = 1;

                for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

                    node.len += 1;
                }

                next = &mut node.next;
            }
        }
    }
}

impl<'cur, T> Iterator for Cursor<'cur, T>
where
    T: 'cur,
{
    type Item = &'cur mut T;

    fn next(&mut self) -> Option<&'cur mut T> {
        let node = Node::as_mut((*self.cur)?);

        let item = unsafe { node.data.get_unchecked_mut(self.idx).assume_init_mut() };

        self.idx += 1;

        if self.idx == PAGE_SIZE {
            self.idx = 0;
            self.cur = &mut node.next;
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

        assert!(list.first.is_none());
    }

    #[test]
    fn one_page() {
        let list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(n));

        let first = Node::as_mut(list.first.unwrap());

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[15].assume_init_read(), 15);
        }
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..3, |n, p| p.put(n));

        let first = Node::as_mut(list.first.unwrap());

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[1].assume_init_read(), 1);
            assert_eq!(first.data[2].assume_init_read(), 2);
        }
    }

    #[test]
    fn two_nodes_with_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.first.unwrap());
            let second = Node::as_mut(first.next.unwrap());

            assert_eq!(**first.data[0].assume_init_ref(), 0);
            assert_eq!(**first.data[15].assume_init_ref(), 15);

            assert_eq!(**second.data[0].assume_init_ref(), 16);
            assert_eq!(**second.data[3].assume_init_ref(), 19);
        }
    }

    #[test]
    fn cursor_iter() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(n));

        for (left, right) in list.cursor().zip(0..100) {
            assert_eq!(*left, right);
        }
    }

    #[test]
    fn cursor_truncate_empty() {
        let mut list = LinkedList::build([], |n: usize, p| p.put(Box::new(n)));

        list.cursor().truncate_rest();
    }

    #[test]
    fn cursor_truncate_one_page() {
        let mut list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(Box::new(n)));

        list.cursor().truncate_rest();
    }

    #[test]
    fn cursor_truncate_many_pages() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        assert!(list.first.is_some());

        list.cursor().truncate_rest();

        assert!(list.first.is_none());
    }

    #[test]
    fn cursor_truncate_many_pages_partial() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        // assert_eq!(list.len, PAGE_SIZE * 3);

        let mut cur = list.cursor();
        cur.by_ref().take(24).count();
        cur.truncate_rest();

        // assert_eq!(list.len, 24);
    }

    #[test]
    fn cursor_truncate_many_pages_partial_at_boundary() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 5, |n, p| p.put(Box::new(n)));

        // assert_eq!(list.len, PAGE_SIZE * 5);

        let mut cur = list.cursor();
        cur.by_ref().take(PAGE_SIZE * 2).count();
        cur.truncate_rest();

        // assert_eq!(list.len, PAGE_SIZE * 2);
    }

    #[test]
    fn cursor_truncate_unaligned() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(Box::new(n)));

        // assert_eq!(list.len, 100);

        let mut cur = list.cursor();

        cur.by_ref().take(50).count();
        cur.truncate_rest();

        // assert_eq!(list.len, 50);
    }

    #[test]
    fn cursor_truncate_extend_unaligned() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(Box::new(n)));

        let mut cur = list.cursor();

        cur.by_ref().take(50).count();
        cur.truncate_rest()
            .extend(200..250, |n, p| p.put(Box::new(n)));

    }
}
