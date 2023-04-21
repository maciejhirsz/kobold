use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

const PAGE_SIZE: usize = 16;

struct Node<T> {
    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    meta: Meta<T>,

    /// Array to store all the elements of the `Node` in
    data: [MaybeUninit<T>; PAGE_SIZE],
}

#[derive(Clone, Copy)]
union Meta<T> {
    len: usize,
    next: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of nodes in the list
    nodes: usize,

    /// First `Node` in the list
    first: Meta<T>,
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

    unsafe fn assume_slice(&mut self) -> &mut [T] {
        let len = unsafe { self.meta.len };
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
        let mut first = Meta {
            next: NonNull::dangling(),
        };

        let mut iter = iter.into_iter();
        let mut nodes = 0;

        unsafe {
            let mut next = &mut first.next;

            while let Some(item) = iter.next() {
                *next = Node::new();

                let node = Node::as_mut(*next);

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                node.meta.len = 1;

                for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

                    node.meta.len += 1;
                }

                next = &mut node.meta.next;
                nodes += 1;
            }
        }

        LinkedList { nodes, first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            rem: self.nodes,
            cur: &mut self.first,
            nodes: &mut self.nodes,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut node;

        unsafe {
            let mut next = self.first.next;

            if self.nodes > 1 {
                while self.nodes > 1 {
                    node = Node::as_mut(next);

                    drop_in_place(node.assume_page());

                    next = node.meta.next;
                    self.nodes -= 1;

                    Node::dealloc(node.into());
                }

                Node::as_mut(next).meta.len = 16;
            }

            if self.nodes > 0 {
                node = Node::as_mut(next);

                drop_in_place(node.assume_slice());

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    rem: usize,
    cur: &'cur mut Meta<T>,
    nodes: &'cur mut usize,
}

pub struct Tail<'cur, T> {
    cur: &'cur mut Meta<T>,
    nodes: &'cur mut usize,
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    pub fn truncate_rest(self) -> Tail<'cur, T> {
        if !self.has_next() {
            return Tail {
                cur: self.cur,
                nodes: self.nodes,
            };
        }
        let node = unsafe { self.cur.next };

        unsafe {
            if self.rem > 1 {
                drop(LinkedList {
                    nodes: self.rem - 1,
                    first: Meta {
                        next: Node::as_mut(node).meta.next,
                    },
                });

                Node::as_mut(node).meta.len = 16;
            }

            drop_in_place(&mut Node::as_mut(node).assume_slice()[self.idx..]);

            if self.idx == 0 {
                Node::dealloc(node);
                *self.nodes -= self.rem;

                if *self.nodes == 0 {
                    self.cur.next = NonNull::dangling();
                }
            } else {
                Node::as_mut(node).meta.len = self.idx;
                *self.nodes -= self.rem - 1;
            }
        }

        Tail {
            cur: self.cur,
            nodes: self.nodes,
        }
    }

    pub fn has_next(&self) -> bool {
        self.rem > 0
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

// impl<'cur, T> Tail<'cur, T> {
//     pub fn extend<I, U, F>(self, iter: I, mut constructor: F)
//     where
//         I: IntoIterator<Item = U>,
//         F: FnMut(U, In<T>) -> Out<T>,
//     {
//         let mut iter = iter.into_iter();
//         let mut next = self.cur;
//         let local = *self.len % PAGE_SIZE;

//         unsafe {
//             if local != 0 {
//                 let node = Node::as_mut(*next);

//                 for (slot, item) in node.data[local..].iter_mut().zip(iter.by_ref()) {
//                     In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

//                     *self.len += 1;
//                 }

//                 next = &mut node.next;
//             }

//             while let Some(item) = iter.next() {
//                 *next = Node::new();

//                 let node = Node::as_mut(*next);

//                 In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
//                     constructor(item, p)
//                 });

//                 *self.len += 1;

//                 for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
//                     In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

//                     *self.len += 1;
//                 }

//                 next = &mut node.next;
//             }
//         }
//     }
// }

impl<'cur, T> Iterator for Cursor<'cur, T>
where
    T: 'cur,
{
    type Item = &'cur mut T;

    fn next(&mut self) -> Option<&'cur mut T> {
        if !self.has_next() {
            return None;
        }

        let cur = Node::as_mut(unsafe { self.cur.next });

        let item = unsafe { cur.data.get_unchecked_mut(self.idx).assume_init_mut() };

        self.idx += 1;

        if self.idx == PAGE_SIZE {
            self.idx = 0;
            self.rem -= 1;
            self.cur = &mut cur.meta;
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

        assert_eq!(list.nodes, 0);
    }

    #[test]
    fn one_page() {
        let list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(n));

        assert_eq!(list.nodes, 1);

        unsafe {
            let first = Node::as_mut(list.first.next);

            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[15].assume_init_read(), 15);
        }
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..3, |n, p| p.put(n));

        assert_eq!(list.nodes, 1);

        unsafe {
            let first = Node::as_mut(list.first.next);

            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[1].assume_init_read(), 1);
            assert_eq!(first.data[2].assume_init_read(), 2);
        }
    }

    #[test]
    fn two_nodes_with_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.first.next);
            let second = Node::as_mut(first.meta.next);

            assert_eq!(list.nodes, 2);

            assert_eq!(**first.data[0].assume_init_ref(), 0);
            assert_eq!(**first.data[15].assume_init_ref(), 15);

            assert_eq!(**second.data[0].assume_init_ref(), 16);
            assert_eq!(**second.data[3].assume_init_ref(), 19);
        }
    }

    #[test]
    fn cursor_iter() {
        let mut list = LinkedList::build(0..100, |n, p| p.put(n));

        assert_eq!(list.nodes, 7);

        for (left, right) in list.cursor().zip(0..100) {
            assert_eq!(*left, right);
        }
    }

    #[test]
    fn cursor_truncate_empty() {
        let mut list = LinkedList::build([], |n: usize, p| p.put(Box::new(n)));

        assert_eq!(list.nodes, 0);

        list.cursor().truncate_rest();
    }

    #[test]
    fn cursor_truncate_one_page() {
        let mut list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(Box::new(n)));

        assert_eq!(list.nodes, 1);

        list.cursor().truncate_rest();

        assert_eq!(list.nodes, 0);
    }

    #[test]
    fn cursor_truncate_many_pages() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        assert_eq!(list.nodes, 3);

        list.cursor().truncate_rest();

        assert_eq!(list.nodes, 0);
    }

    #[test]
    fn cursor_truncate_many_pages_partial() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

        assert_eq!(list.nodes, 3);

        let mut cur = list.cursor();
        cur.by_ref().take(24).count();
        cur.truncate_rest();

        assert_eq!(list.nodes, 2);
    }

    #[test]
    fn cursor_truncate_many_pages_partial_at_boundary() {
        let mut list = LinkedList::build(0..PAGE_SIZE * 5, |n, p| p.put(Box::new(n)));

        assert_eq!(list.nodes, 5);

        let mut cur = list.cursor();
        cur.by_ref().take(PAGE_SIZE * 2).count();
        cur.truncate_rest();

        assert_eq!(list.nodes, 2);
    }

    // #[test]
    // fn cursor_truncate_unaligned() {
    //     let mut list = LinkedList::build(0..100, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, 100);

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(50).count();
    //     cur.truncate_rest();

    //     assert_eq!(list.len, 50);
    // }

    // #[test]
    // fn cursor_truncate_extend_unaligned() {
    //     let mut list = LinkedList::build(0..100, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, 100);

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(50).count();
    //     cur.truncate_rest()
    //         .extend(200..250, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, 100);
    // }
}
