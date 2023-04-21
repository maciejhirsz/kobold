use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::NonNull;

use crate::internal::{In, Out};

const PAGE_SIZE: usize = 16;

struct Node<T> {
    /// Array to store all the elements of the `Node` in
    data: [MaybeUninit<T>; PAGE_SIZE],

    /// Length of the elements written, or a pointer to the next `Node`
    meta: Meta<T>,
}

union Meta<T> {
    /// Number of elements written in `Node`
    len: usize,

    /// Pointer to next `Node`, assumes current `Node` is at `PAGE_SIZE`
    next: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of links in this list. This is equivalent to number
    /// of `Node`s minus one, or total number of `Nodes` that have a full
    /// set of `PAGE_SIZE` elements.
    links: usize,

    /// First `Node` in the list
    first: NonNull<Node<T>>,
}

impl<T> Node<T> {
    fn new() -> NonNull<Self> {
        use std::alloc::{alloc, Layout};
        use std::ptr::addr_of_mut;

        unsafe {
            let ptr = alloc(Layout::new::<Self>()) as *mut Self;

            addr_of_mut!((*ptr).meta).write(Meta { len: 0 });

            NonNull::new_unchecked(ptr)
        }
    }

    unsafe fn dealloc(ptr: NonNull<Self>) {
        use std::alloc::{dealloc, Layout};

        dealloc(ptr.as_ptr().cast(), Layout::new::<Self>());
    }

    // unsafe fn assume_page(&mut self) -> &mut [T; PAGE_SIZE] {
    //     &mut *(&mut self.data as *mut _ as *mut [T; PAGE_SIZE])
    // }

    // unsafe fn assume_tail(&mut self) -> &mut [T; PAGE_SIZE] {
    //     &mut *(&mut self.data[..self.meta.len] as *mut _ as *mut [T])
    // }

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
        let mut links = 0;

        unsafe {
            loop {
                for item in iter.by_ref().take(PAGE_SIZE) {
                    In::pinned(Pin::new_unchecked(&mut node.data[node.meta.len]), |p| {
                        constructor(item, p)
                    });
                    node.meta.len += 1;
                }

                if node.meta.len < PAGE_SIZE {
                    break;
                }

                links += 1;
                node.meta.next = Node::new();
                node = Node::as_mut(node.meta.next);
            }
        }

        LinkedList { links, first }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut node;

        unsafe {
            while self.links > 0 {
                node = Node::as_mut(self.first);

                assume_drop_page(&mut node.data);

                self.first = node.meta.next;
                self.links -= 1;

                Node::dealloc(node.into());
            }

            node = Node::as_mut(self.first);

            assume_drop_slice(&mut node.data[..node.meta.len]);

            Node::dealloc(node.into());
        }
    }
}

unsafe fn assume_drop_page<T>(slice: *mut [MaybeUninit<T>; PAGE_SIZE]) {
    std::ptr::drop_in_place(slice as *mut [T; PAGE_SIZE])
}

unsafe fn assume_drop_slice<T>(slice: *mut [MaybeUninit<T>]) {
    std::ptr::drop_in_place(slice as *mut [T])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_node() {
        let list = LinkedList::build([42, 100, 404], |n, p| p.put(n));

        assert_eq!(list.links, 0);

        let first = Node::as_mut(list.first);

        unsafe {
            assert_eq!(first.meta.len, 3);
        }
    }

    #[test]
    fn two_nodes_with_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(format!("{n}")));

        unsafe {
            let first = Node::as_mut(list.first);
            let second = Node::as_mut(first.meta.next);

            assert_eq!(first.data[0].assume_init_mut(), "0");
            assert_eq!(first.data[15].assume_init_mut(), "15");

            assert_eq!(second.meta.len, 4);
            assert_eq!(second.data[0].assume_init_mut(), "16");
            assert_eq!(second.data[3].assume_init_mut(), "19");
        }
    }
}
