use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    next: NonNull<Node<T>>,

    /// All the elements of the `Node`
    data: [MaybeUninit<T>],
}

#[repr(C)]
struct Head<T> {
    next: NonNull<Node<T>>,
}

union FatPtr<T> {
    raw: (NonNull<Head<T>>, usize),
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
            std::mem::size_of::<(NonNull<Head<T>>, usize)>(),
            std::mem::size_of::<NonNull<Node<T>>>()
        );

        Vec::<u32>::new().into_boxed_slice();

        unsafe {
            let head = NonNull::new_unchecked(alloc(Self::layout(cap)) as *mut Head<T>);

            FatPtr { raw: (head, cap) }.fat
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

    unsafe fn assume_slice(&mut self, len: usize) -> &mut [T] {
        &mut *(&mut self.data[..len] as *mut _ as *mut [T])
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

impl<T> LinkedList<T> {
    pub fn build<I, U, F>(iter: I, mut constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut first = Node::dangling();

        let mut iter = iter.into_iter();
        let mut next = &mut first;
        let mut len = 0;

        unsafe {
            while let Some(item) = iter.next() {
                *next = Node::new(iter.size_hint().0 + 1);

                let node = Node::as_mut(*next);

                In::pinned(Pin::new_unchecked(&mut node.data[0]), |p| {
                    constructor(item, p)
                });

                len += 1;

                for (slot, item) in node.data[1..].iter_mut().zip(iter.by_ref()) {
                    In::pinned(Pin::new_unchecked(slot), |p| constructor(item, p));

                    len += 1;
                }

                next = &mut node.next;
            }
        }

        LinkedList { len, first }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            cut: 0,
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

                if self.len > node.capacity() {
                    drop_in_place(node.assume_slice(node.capacity()));

                    self.first = node.next;
                    self.len -= node.capacity();
                } else {
                    drop_in_place(node.assume_slice(self.len));
                    self.len = 0;
                }

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    cut: usize,
    cur: &'cur mut NonNull<Node<T>>,
    len: &'cur mut usize,
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    // pub fn truncate_rest(self) -> Tail<'cur, T> {
    //     if self.idx == *self.len {
    //         return Tail {
    //             cur: self.cur,
    //             len: self.len,
    //         };
    //     }

    //     let mut cur = *self.cur;
    //     let mut remain = *self.len - self.idx;
    //     let local = self.idx % PAGE_SIZE;

    //     if local != 0 {
    //         let node = cur;
    //         let mut drop_local = PAGE_SIZE - local;

    //         if drop_local <= remain {
    //             cur = Node::as_mut(cur).next;
    //             remain -= drop_local;
    //         } else {
    //             drop_local = remain;
    //             remain = 0;
    //         };

    //         unsafe {
    //             drop_in_place(&mut Node::as_mut(node).assume_page()[local..local + drop_local]);
    //         }
    //     };

    //     *self.len = self.idx;

    //     drop(LinkedList {
    //         len: remain,
    //         first: cur,
    //     });

    //     Tail {
    //         cur: self.cur,
    //         len: self.len,
    //     }
    // }

    pub fn has_next(&self) -> bool {
        self.idx != *self.len
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
                continue;
            }

            break;
        }
    }
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
            self.idx = 0;
            self.cut += cap;
            self.cur = &mut cur.next;
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
