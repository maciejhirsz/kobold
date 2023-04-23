use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

const PAGE_SIZE: usize = 16;

struct Node<T> {
    /// Array to store all the elements of the `Node` in
    data: [MaybeUninit<T>; PAGE_SIZE],

    /// Pointer to the next `Node`.
    meta: Meta<T>,
}

union Meta<T> {
    next: NonNull<Node<T>>,
    len: usize,
}

impl<T> Meta<T> {
    fn next(&mut self) -> Option<NonNull<Node<T>>> {
        unsafe {
            if self.len > 16 {
                Some(self.next)
            } else {
                None
            }
        }
    }

    fn len(&self) -> usize {
        unsafe { std::cmp::min(self.len, PAGE_SIZE) }
    }
}

pub struct LinkedList<T> {
    /// First `Node` in the list
    meta: Meta<T>,
}

impl<T> Node<T> {
    fn new() -> NonNull<Self> {
        use std::alloc::{alloc, Layout};

        unsafe {
            let node = alloc(Layout::new::<Self>()) as *mut Self;

            std::ptr::addr_of_mut!((*node).meta).write(Meta { len: 0 });

            NonNull::new_unchecked(node)
        }
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
    pub fn build<I, U, F>(iter: I, constructor: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut meta = Meta { len: 0 };

        Tail { meta: &mut meta }.extend(iter, constructor);

        LinkedList { meta }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            idx: 0,
            len: self.meta.len(),
            meta: &mut self.meta,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        unsafe {
            while let Some(node) = self.meta.next() {
                let node = Node::as_mut(node);

                drop_in_place(node.assume_slice(node.meta.len()));

                self.meta.next = node.meta.next;

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    len: usize,
    meta: &'cur mut Meta<T>,
}

pub struct Tail<'cur, T> {
    meta: &'cur mut Meta<T>,
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
//     pub fn truncate_rest(self) -> Tail<'cur, T> {
//         if self.idx == *self.len || self.cur.is_none() {
//             return Tail {
//                 cur: self.cur,
//                 len: self.len,
//             };
//         }

//         let node = Node::as_mut(self.cur.unwrap());
//         let local = self.idx % PAGE_SIZE;
//         let remain = *self.len - self.idx;

//         let mut drop_local = PAGE_SIZE - local;

//         if drop_local < remain {
//             drop(LinkedList {
//                 len: remain - drop_local,
//                 first: node.next.take(),
//             })
//         } else {
//             drop_local = remain;
//         }

//         unsafe {
//             drop_in_place(&mut node.assume_page()[local..local + drop_local]);
//         }

//         *self.len = self.idx;

//         Tail {
//             cur: self.cur,
//             len: self.len,
//         }
//     }

    pub fn has_next(&self) -> bool {
        self.idx != self.len
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

impl<'cur, T> Tail<'cur, T> {
    pub fn extend<I, U, F>(self, iter: I, mut constructor: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut meta = self.meta;

        unsafe {
            for item in iter {
                let node = {
                    if meta.len <= 16 {
                        meta.next = Node::new();
                    }
                    Node::as_mut(meta.next)
                };

                In::pinned(Pin::new_unchecked(node.data.get_unchecked_mut(node.meta.len)), |p| {
                    constructor(item, p)
                });

                node.meta.len += 1;

                if node.meta.len == PAGE_SIZE {
                    meta = &mut node.meta;
                }
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
        unsafe {
            if self.idx == self.len {
                return None;
            }

            let cur = Node::as_mut(self.meta.next);

            let item = cur.data.get_unchecked_mut(self.idx).assume_init_mut();

            self.idx += 1;

            if self.idx == self.len {
                self.idx = 0;
                self.meta = &mut cur.meta;
                self.len = self.meta.len();
            }

            Some(item)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list() {
        let list = LinkedList::build([], |n: usize, p| p.put(n));

        unsafe {
            assert_eq!(list.meta.len, 0);
        }
    }

    #[test]
    fn one_page() {
        let list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(n));

        unsafe {
            let first = Node::as_mut(list.meta.next);

            assert_eq!(first.meta.len, PAGE_SIZE);
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[15].assume_init_read(), 15);
        }
    }

    #[test]
    fn one_node() {
        let list = LinkedList::build(0..3, |n, p| p.put(n));

        unsafe {
            let first = Node::as_mut(list.meta.next);

            assert_eq!(first.meta.len, 3);
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[1].assume_init_read(), 1);
            assert_eq!(first.data[2].assume_init_read(), 2);
        }
    }

    #[test]
    fn two_nodes_with_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.meta.next);
            let second = Node::as_mut(first.meta.next);

            assert_eq!(second.meta.len, 20 - PAGE_SIZE);

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

    // #[test]
    // fn cursor_truncate_empty() {
    //     let mut list = LinkedList::build([], |n: usize, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, 0);

    //     list.cursor().truncate_rest();
    // }

    // #[test]
    // fn cursor_truncate_one_page() {
    //     let mut list = LinkedList::build(0..PAGE_SIZE, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, PAGE_SIZE);

    //     list.cursor().truncate_rest();
    // }

    // #[test]
    // fn cursor_truncate_many_pages() {
    //     let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, PAGE_SIZE * 3);

    //     list.cursor().truncate_rest();

    //     assert_eq!(list.len, 0);
    // }

    // #[test]
    // fn cursor_truncate_many_pages_partial() {
    //     let mut list = LinkedList::build(0..PAGE_SIZE * 3, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, PAGE_SIZE * 3);

    //     let mut cur = list.cursor();
    //     cur.by_ref().take(24).count();
    //     cur.truncate_rest();

    //     assert_eq!(list.len, 24);
    // }

    // #[test]
    // fn cursor_truncate_many_pages_partial_at_boundary() {
    //     let mut list = LinkedList::build(0..PAGE_SIZE * 5, |n, p| p.put(Box::new(n)));

    //     assert_eq!(list.len, PAGE_SIZE * 5);

    //     let mut cur = list.cursor();
    //     cur.by_ref().take(PAGE_SIZE * 2).count();
    //     cur.truncate_rest();

    //     assert_eq!(list.len, PAGE_SIZE * 2);
    // }

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

    //     for (left, right) in list.cursor().zip((0..50).chain(200..250)) {
    //         assert_eq!(**left, right);
    //     }
    // }
}
