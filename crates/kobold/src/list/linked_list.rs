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
}
