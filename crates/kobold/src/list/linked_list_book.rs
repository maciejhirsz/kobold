use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    next: Page<T>,

    /// All the elements of the `Node`
    data: [MaybeUninit<T>],
}

struct Page<T> {
    node: NonNull<Node<T>>,
    len: usize,
}

impl<T> Clone for Page<T> {
    fn clone(&self) -> Self {
        Page {
            node: self.node,
            len: self.len,
        }
    }
}

impl<T> Copy for Page<T> {}

impl<T> Page<T> {
    fn empty() -> Self {
        Page {
            node: unsafe { FatPtr { raw: (NonNull::dangling(), 0) }.fat },
            len: 0,
        }
    }

    fn node<'a>(&self) -> &'a mut Node<T> {
        Node::as_mut(self.node)
    }

    fn capacity(&self) -> usize {
        unsafe { FatPtr { fat: self.node }.raw.1 }
    }

    fn is_full(&self) -> bool {
        self.len == self.capacity()
    }
}

#[repr(C)]
struct Head<T> {
    next: Page<T>,
}

union FatPtr<T> {
    raw: (NonNull<Head<T>>, usize),
    fat: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// First `Node` in the list
    first: Page<T>,
}

impl<T> Node<T> {
    const MIN_PAGE_SIZE: usize = {
        let n = 256 / std::mem::size_of::<T>();

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
            let head = alloc(Self::layout(cap)) as *mut Head<T>;

            head.write(Head {
                next: Page::empty(),
            });

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

    unsafe fn assume_page(&mut self) -> &mut [T] {
        &mut *(&mut self.data as *mut _ as *mut [T])
    }

    unsafe fn assume_slice(&mut self, len: usize) -> &mut [T] {
        &mut *(self.data.get_unchecked_mut(..len) as *mut _ as *mut [T])
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
        let mut first = Page::empty();

        let mut iter = iter.into_iter();
        let mut page = &mut first;

        while let Some(item) = iter.next() {
            if page.is_full() {
                *page = Page {
                    node: Node::new(iter.size_hint().0 + 1),
                    len: 0,
                };
            }

            In::pinned(
                unsafe { Pin::new_unchecked(page.node().data.get_unchecked_mut(page.len)) },
                |p| constructor(item, p),
            );

            page.len += 1;

            if page.is_full() {
                page = &mut page.node().next;
            }
        }

        LinkedList { first }
    }

    // pub fn cursor(&mut self) -> Cursor<T> {
    //     Cursor {
    //         idx: 0,
    //         cur: &mut self.first,
    //     }
    // }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        unsafe {
            while self.first.capacity() > 0 {
                drop_in_place(self.first.node().assume_slice(self.first.len));

                let node = self.first.node;
                self.first = self.first.node().next;

                Node::dealloc(node);
            }
        }
    }
}

// pub struct Cursor<'cur, T> {
//     idx: usize,
//     cur: &'cur mut Option<NonNull<Node<T>>>,
// }

// pub struct Tail<'cur, T> {
//     idx: usize,
//     cur: &'cur mut Option<NonNull<Node<T>>>,
// }

// impl<'cur, T> Tail<'cur, T> {
//     pub fn extend<I, U, F>(mut self, iter: I, mut constructor: F)
//     where
//         I: IntoIterator<Item = U>,
//         F: FnMut(U, In<T>) -> Out<T>,
//     {
//         let mut iter = iter.into_iter();
//         let mut next = self.cur;

//         while let Some(item) = iter.next() {
//             let node = Node::as_mut(*next.get_or_insert_with(|| Node::new(iter.size_hint().0 + 1)));

//             In::pinned(
//                 unsafe { Pin::new_unchecked(node.data.get_unchecked_mut(self.idx)) },
//                 |p| constructor(item, p),
//             );

//             self.idx += 1;

//             if self.idx == node.capacity() {
//                 self.idx = 0;
//                 next = &mut node.next;
//             }
//         }
//     }
// }

// impl<'cur, T> Cursor<'cur, T>
// where
//     T: 'cur,
// {
//     pub fn truncate_rest(self) -> Tail<'cur, T> {
//         if self.cur.is_none() {
//             return Tail {
//                 idx: self.idx,
//                 cur: self.cur,
//             };
//         }

//         let node = Node::as_mut(self.cur.unwrap());

//         drop(LinkedList {
//             first: node.next.take(),
//         });

//         let len = node.len;

//         unsafe {
//             drop_in_place(node.assume_page().get_unchecked_mut(self.idx..len));
//         }

//         node.len = self.idx;

//         Tail {
//             idx: self.idx,
//             cur: self.cur,
//         }
//     }

//     pub fn pair<I, F, U>(&mut self, iter: I, mut each: F)
//     where
//         I: IntoIterator<Item = U>,
//         F: FnMut(&mut T, U),
//     {
//         let mut iter = iter.into_iter();

//         while let Some(cur) = self.cur.map(Node::as_mut) {
//             if let Some(item) = iter.next() {
//                 let slot = unsafe { cur.data.get_unchecked_mut(self.idx).assume_init_mut() };

//                 each(slot, item);

//                 self.idx += 1;

//                 if self.idx == cur.len {
//                     self.idx = 0;
//                     self.cur = &mut cur.next;
//                 }
//                 continue;
//             }

//             break;
//         }
//     }
// }

// impl<'cur, T> Iterator for Cursor<'cur, T>
// where
//     T: 'cur,
// {
//     type Item = &'cur mut T;

//     fn next(&mut self) -> Option<&'cur mut T> {
//         let cur = self.cur.map(Node::as_mut)?;

//         let item = unsafe { cur.data.get_unchecked_mut(self.idx).assume_init_mut() };

//         self.idx += 1;

//         if self.idx == cur.len {
//             self.idx = 0;
//             self.cur = &mut cur.next;
//         }

//         Some(item)
//     }
// }

#[cfg(test)]
mod tests {
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
        let list = LinkedList::build([], |n: usize, p| p.put(n));

        assert!(list.first.is_full());
        assert_eq!(list.first.capacity(), 0);
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..128, |n, p| p.put(n));

        let first = list.first;

        unsafe {
            assert_eq!(first.node().data[0].assume_init_read(), 0);
            assert_eq!(first.node().data[127].assume_init_read(), 127);
        }
    }

    // #[test]
    // fn one_node_alloc() {
    //     let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

    //     unsafe {
    //         let first = list.first;

    //         assert_eq!(**first.node().data[0].assume_init_ref(), 0);
    //         assert_eq!(**first.node().data[19].assume_init_ref(), 19);
    //     }
    // }

    // #[test]
    // fn many_nodes() {
    //     let mut list = LinkedList::build(NoHint(0..128), |n, p| p.put(n));

    //     let first = list.first;

    //     assert!(first.len < 128);

    //     for (left, right) in list.cursor().zip(0..128) {
    //         assert_eq!(*left, right);
    //     }
    // }

    // #[test]
    // fn cursor_iter() {
    //     let mut list = LinkedList::build(0..100, |n, p| p.put(n));

    //     for (left, right) in list.cursor().zip(0..100) {
    //         assert_eq!(*left, right);
    //     }
    // }

    // #[test]
    // fn cursor_truncate_unaligned() {
    //     let mut list = LinkedList::build(NoHint(0..200), |n, p| p.put(Box::new(n)));

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(100).count();
    //     cur.truncate_rest();

    //     for (left, right) in list.cursor().zip(0..100) {
    //         assert_eq!(**left, right);
    //     }
    // }

    // #[test]
    // fn cursor_truncate_extend_unaligned() {
    //     let mut list = LinkedList::build(NoHint(0..200), |n, p| p.put(Box::new(n)));

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(100).count();
    //     cur.truncate_rest()
    //         .extend(200..300, |n, p| p.put(Box::new(n)));

    //     for (left, right) in list.cursor().zip((0..100).chain(200..300)) {
    //         assert_eq!(**left, right);
    //     }
    // }

    // #[test]
    // fn cursor_truncate_extend_empty() {
    //     let mut list = LinkedList::build(NoHint(0..200), |n, p| p.put(Box::new(n)));

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(100).count();
    //     cur.truncate_rest().extend([], |n, p| p.put(Box::new(n)));

    //     for (left, right) in list.cursor().zip(0..100) {
    //         assert_eq!(**left, right);
    //     }
    // }

    // #[test]
    // fn cursor_truncate_extend_aligned() {
    //     let mut list = LinkedList::build(NoHint(0..256), |n, p| p.put(Box::new(n)));

    //     let mut cur = list.cursor();

    //     cur.by_ref().take(128).count();
    //     cur.truncate_rest()
    //         .extend(512..640, |n, p| p.put(Box::new(n)));

    //     for (left, right) in list.cursor().zip((0..128).chain(512..640)) {
    //         assert_eq!(**left, right);
    //     }
    // }
}
