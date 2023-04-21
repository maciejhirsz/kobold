use std::mem::MaybeUninit;
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

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }
}

impl<T> LinkedList<T> {
    pub fn build<I, U, F>(iter: I, constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let first = Node::new();

        let mut node = first;
        let mut links = 0;

        LinkedList { links, first }
    }
}
