use std::cmp;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

const PAGE_SIZE: usize = 16;

struct Node<T> {
    /// Array to store all the elements of the `Node` in
    data: [MaybeUninit<T>; PAGE_SIZE],

    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    next: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of elements in the list
    len: usize,

    /// First `Node` in the list
    first: NonNull<Node<T>>,
}

impl<T> Node<T> {
    fn new() -> NonNull<Self> {
        use std::alloc::{alloc, Layout};

        unsafe { NonNull::new_unchecked(alloc(Layout::new::<Self>()) as *mut Self) }
    }

    unsafe fn dealloc(ptr: NonNull<Self>) {
        use std::alloc::{dealloc, Layout};

        dealloc(ptr.as_ptr().cast(), Layout::new::<Self>());
    }

    unsafe fn assume_page(&mut self) -> &mut [T; PAGE_SIZE] {
        &mut *(&mut self.data as *mut _ as *mut [T; PAGE_SIZE])
    }

    unsafe fn assume_slice(&mut self, len: usize) -> &mut [T] {
        &mut *(&mut self.data[..len] as *mut _ as *mut [T])
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
        let mut first = NonNull::dangling();

        let mut iter = iter.into_iter();
        let mut next = &mut first;
        let mut len = 0;

        unsafe {
            while let Some(item) = iter.next() {
                *next = Node::new();

                let node = Node::as_mut(*next);

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                len += 1;

                for item in iter.by_ref().take(PAGE_SIZE - 1) {
                    In::pinned(Pin::new_unchecked(&mut node.data[len % PAGE_SIZE]), |p| {
                        constructor(item, p)
                    });

                    len += 1;
                }

                next = &mut node.next;
            }
        }

        LinkedList { len, first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            cur: (self.len > 0).then_some(self.first),
            idx: 0,
            ll: self,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut node;

        unsafe {
            while self.len > PAGE_SIZE {
                node = Node::as_mut(self.first);

                drop_in_place(node.assume_page());

                self.first = node.next;
                self.len -= PAGE_SIZE;

                Node::dealloc(node.into());
            }

            if self.len > 0 {
                node = Node::as_mut(self.first);

                drop_in_place(node.assume_slice(self.len));

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    cur: Option<NonNull<Node<T>>>,
    idx: usize,
    ll: &'cur mut LinkedList<T>,
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    pub fn truncate_rest(self) {
        let mut cur = match self.cur {
            Some(cur) => cur,
            None => return,
        };

        let mut remain = self.ll.len - self.idx;
        let local = self.idx % PAGE_SIZE;

        if local != 0 {
            let node = cur;
            let mut drop_local = PAGE_SIZE - local;

            if drop_local <= remain {
                cur = Node::as_mut(cur).next;
                remain -= drop_local;
            } else {
                drop_local = remain;
                remain = 0;
            };

            unsafe {
                drop_in_place(&mut Node::as_mut(node).assume_page()[local..local + drop_local]);
            }
        };

        self.ll.len = self.idx;

        drop(LinkedList {
            len: remain,
            first: cur,
        });
    }
}

impl<'cur, T> Iterator for Cursor<'cur, T>
where
    T: 'cur,
{
    type Item = &'cur mut T;

    fn next(&mut self) -> Option<&'cur mut T> {
        let cur = self.cur.map(Node::as_mut)?;

        let local = self.idx % PAGE_SIZE;
        let item = unsafe { cur.data[local].assume_init_mut() };

        if local == PAGE_SIZE - 1 {
            self.cur = (self.ll.len != self.idx).then_some(cur.next);
        }

        self.idx += 1;

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
    fn one_page() {
        let list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(n));

        assert_eq!(list.len, PAGE_SIZE);

        let first = Node::as_mut(list.first);

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[15].assume_init_read(), 15);
        }
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..3, |n, p| p.put(n));

        assert_eq!(list.len, 3);

        let first = Node::as_mut(list.first);

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
            let first = Node::as_mut(list.first);
            let second = Node::as_mut(first.next);

            assert_eq!(list.len, 20);

            assert_eq!(**first.data[0].assume_init_ref(), 0);
            assert_eq!(**first.data[15].assume_init_ref(), 15);

            assert_eq!(**second.data[0].assume_init_ref(), 16);
            assert_eq!(**second.data[3].assume_init_ref(), 19);
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

    #[test]
    fn cursor_truncate_empty() {
        let mut list = LinkedList::build([], |n: usize, p| p.put(Box::new(n)));

        assert_eq!(list.len, 0);

        list.cursor().truncate_rest();
    }

    #[test]
    fn cursor_truncate_one_page() {
        let mut list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(Box::new(n)));

        assert_eq!(list.len, PAGE_SIZE);

        list.cursor().truncate_rest();
    }

    #[test]
    fn cursor_truncate_many_pages() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        assert_eq!(list.len, PAGE_SIZE * 3);

        list.cursor().truncate_rest();

        assert_eq!(list.len, 0);
    }

    #[test]
    fn cursor_truncate_many_pages_partial() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        assert_eq!(list.len, PAGE_SIZE * 3);

        let mut cur = list.cursor();
        cur.by_ref().take(24).count();
        cur.truncate_rest();

        assert_eq!(list.len, 24);
    }

    #[test]
    fn cursor_truncate_many_pages_partial_at_boundary() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 5, |n, p| p.put(Box::new(n)));

        assert_eq!(list.len, PAGE_SIZE * 5);

        let mut cur = list.cursor();
        cur.by_ref().take(PAGE_SIZE * 2).count();
        cur.truncate_rest();

        assert_eq!(list.len, PAGE_SIZE * 2);
    }

    #[test]
    fn cursor_truncate_unaligned() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(Box::new(n)));

        assert_eq!(list.len, 100);

        let mut cur = list.cursor();

        cur.by_ref().take(50).count();
        cur.truncate_rest();

        assert_eq!(list.len, 50);
    }
}
