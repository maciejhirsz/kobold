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
        let first = Node::new();

        let mut iter = iter.into_iter();
        let mut node = Node::as_mut(first);
        let mut len = 0;

        unsafe {
            loop {
                for (item, slot) in iter.by_ref().take(PAGE_SIZE).zip(&mut node.data) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));
                    len += 1;
                }

                if (len % PAGE_SIZE) > 0 {
                    break;
                }

                node.next = Node::new();
                node = Node::as_mut(node.next);
            }
        }

        LinkedList { len, first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            cur: self.first,
            idx: 0,
            ll: self,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut node;

        unsafe {
            for _ in 0..self.len / PAGE_SIZE {
                node = Node::as_mut(self.first);

                drop_in_place(node.assume_page());

                self.first = node.next;

                Node::dealloc(node.into());
            }

            node = Node::as_mut(self.first);

            drop_in_place(node.assume_slice(self.len % PAGE_SIZE));

            Node::dealloc(node.into());
        }
    }
}

pub struct Cursor<'cur, T> {
    cur: NonNull<Node<T>>,
    idx: usize,
    ll: &'cur mut LinkedList<T>,
}

// impl<'cur, T> Cursor<'cur, T>
// where
//     T: 'cur,
// {
//     pub fn truncate_rest(self) {
//         if self.idx == self.ll.len {
//             return;
//         }
//         let len = self.ll.len;
//         self.ll.len = self.idx;

//         let node = Node::as_mut(self.cur);
//         let local = self.idx % PAGE_SIZE;
//         let mut remain = len - self.idx;

//         let tail_list = if local == 0 {
//             Some(self.cur)
//         } else if let Some(slice) = remain.checked_sub(local) {
//             Some(node.next)
//         } else {
//             None
//         };

//         let len = self.ll.len;
//         self.ll.len = self.idx;

//         // let node = Node::as_mut(self.cur);
//         let mut remain = len - self.idx;

//         match remain + local {
//             0 => {
//                 // do nothing
//             }
//             1..15 => {
//                 // drop current
//             }
//         }

//         unsafe {
//             if self.idx != 0 && local == 0 {
//                 //
//             } else {

//             }

//             if local > 0 {
//                 let to_drop = local..cmp::min(local + remain, PAGE_SIZE);
//                 drop_in_place(&mut node.assume_page()[to_drop]);

//                 remain -= local;
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
        if self.idx == self.ll.len {
            return None;
        }

        let local = self.idx % PAGE_SIZE;

        if self.idx != 0 && local == 0 {
            self.cur = Node::as_mut(self.cur).next;
        }

        self.idx += 1;

        Some(unsafe { Node::as_mut(self.cur).data[local].assume_init_mut() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_node() {
        let list = LinkedList::build([42, 100, 404], |n, p| p.put(n));

        assert_eq!(list.len, 3);

        let first = Node::as_mut(list.first);

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 42);
            assert_eq!(first.data[1].assume_init_read(), 100);
            assert_eq!(first.data[2].assume_init_read(), 404);
        }
    }

    #[test]
    fn two_nodes_with_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(format!("{n}")));

        unsafe {
            let first = Node::as_mut(list.first);
            let second = Node::as_mut(first.next);

            assert_eq!(list.len, 20);

            assert_eq!(first.data[0].assume_init_mut(), "0");
            assert_eq!(first.data[15].assume_init_mut(), "15");

            assert_eq!(second.data[0].assume_init_mut(), "16");
            assert_eq!(second.data[3].assume_init_mut(), "19");
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

    // #[test]
    // fn cursor_truncate() {
    //     let mut list = LinkedList::build(0..100, |n, p| p.put(n));

    //     assert_eq!(list.len, 100);

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(50).count();
    //     cur.truncate_rest();

    //     assert_eq!(list.len, 50);
    // }
}
