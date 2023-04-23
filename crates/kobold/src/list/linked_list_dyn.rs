use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::{drop_in_place, NonNull};

use crate::internal::{In, Out};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node`. If this is a tail node,
    /// the address of this pointer will be uninitialized junk.
    next: Option<NonNull<Node<T>>>,

    /// All the elements of the `Node`
    data: [MaybeUninit<T>],
}

#[repr(C)]
struct Head<T> {
    next: Option<NonNull<Node<T>>>,
}

union FatPtr<T> {
    raw: (NonNull<Head<T>>, usize),
    fat: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of elements in the list
    len: usize,

    /// First `Node` in the list
    first: Option<NonNull<Node<T>>>,
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
            let head = alloc(Self::layout(cap)) as *mut Head<T>;

            std::ptr::addr_of_mut!((*head).next).write(None);

            FatPtr { raw: (NonNull::new_unchecked(head), cap) }.fat
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
        let mut first = None;

        let mut iter = iter.into_iter();
        let mut next = &mut first;
        let mut len = 0;
        let mut node_len = 0;

        while let Some(item) = iter.next() {
            let node = Node::as_mut(*next.get_or_insert_with(|| Node::new(iter.size_hint().0 + 1)));

            In::pinned(
                unsafe { Pin::new_unchecked(node.data.get_unchecked_mut(node_len)) },
                |p| constructor(item, p),
            );

            node_len += 1;

            if node_len == node.capacity() {
                len += node_len;
                node_len = 0;
                next = &mut node.next;
            }
        }

        len += node_len;

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
        unsafe {
            while let Some(node) = self.first {
                let node = Node::as_mut(node);

                drop_in_place(node.assume_slice(std::cmp::min(self.len, node.capacity())));

                self.first = node.next;
                self.len = self.len.wrapping_sub(node.capacity());

                Node::dealloc(node.into());
            }
        }
    }
}

pub struct Cursor<'cur, T> {
    idx: usize,
    cut: usize,
    cur: &'cur mut Option<NonNull<Node<T>>>,
    len: &'cur mut usize,
}

pub struct Tail<'cur, T> {
    cur: &'cur mut Option<NonNull<Node<T>>>,
    len: &'cur mut usize,
}


impl<'cur, T> Tail<'cur, T> {
    pub fn extend<I, U, F>(self, iter: I, mut constructor: F)
    where
        I: IntoIterator<Item = U>,
        F: FnMut(U, In<T>) -> Out<T>,
    {
        let mut iter = iter.into_iter();
        let mut next = self.cur;
        let mut node_len = 0;

        while let Some(item) = iter.next() {
            let node = Node::as_mut(*next.get_or_insert_with(|| Node::new(iter.size_hint().0 + 1)));

            In::pinned(
                unsafe { Pin::new_unchecked(node.data.get_unchecked_mut(node_len)) },
                |p| constructor(item, p),
            );

            node_len += 1;

            if node_len == node.capacity() {
                *self.len += node_len;
                node_len = 0;
                next = &mut node.next;
            }
        }

        *self.len += node_len;
    }
}

impl<'cur, T> Cursor<'cur, T>
where
    T: 'cur,
{
    pub fn truncate_rest(self) -> Tail<'cur, T> {
        if self.idx == *self.len || self.cur.is_none() {
            return Tail {
                cur: self.cur,
                len: self.len,
            };
        }

        let node = Node::as_mut(self.cur.unwrap());
        let local = self.idx - self.cut;
        let remain = *self.len - self.idx;

        let mut drop_local = node.data.len() - local;

        if drop_local < remain {
            drop(LinkedList {
                len: remain - drop_local,
                first: node.next.take(),
            })
        } else {
            drop_local = remain;
        }

        unsafe {
            drop_in_place(&mut node.assume_page()[local..local + drop_local]);
        }

        *self.len = self.idx;

        Tail {
            cur: self.cur,
            len: self.len,
        }
    }

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
        if self.idx == *self.len {
            return None;
        }

        let cur = Node::as_mut(unsafe { self.cur.unwrap_unchecked() });
        let cap = cur.capacity();

        let local = self.idx - self.cut;
        let item = unsafe { cur.data.get_unchecked_mut(local).assume_init_mut() };

        if local == cap - 1 {
            self.cut += cap;
            self.cur = &mut cur.next;
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
    fn one_node() {
        let list = LinkedList::build(0..128, |n, p| p.put(n));

        assert_eq!(list.len, 128);

        let first = Node::as_mut(list.first.unwrap());

        unsafe {
            assert_eq!(first.data[0].assume_init_read(), 0);
            assert_eq!(first.data[127].assume_init_read(), 127);
        }
    }

    #[test]
    fn one_node_alloc() {
        let list = LinkedList::build(0..20, |n, p| p.put(Box::new(n)));

        unsafe {
            let first = Node::as_mut(list.first.unwrap());

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
