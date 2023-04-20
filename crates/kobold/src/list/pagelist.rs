use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;

use crate::internal::{In, Out};

#[repr(C)]
struct Page<T> {
    next: Option<NonNull<Page<T>>>,
    len: usize,
    data: [MaybeUninit<T>],
}

#[repr(C)]
struct Head<T> {
    next: Option<NonNull<Page<T>>>,
    len: usize,
}

union FatPtr<T> {
    raw: (NonNull<Head<T>>, usize),
    fat: NonNull<Page<T>>,
}

impl<T> Page<T> {
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
            std::mem::size_of::<NonNull<Page<T>>>()
        );

        Vec::<u32>::new().into_boxed_slice();

        unsafe {
            let head = NonNull::new_unchecked(alloc(Self::layout(cap)) as *mut Head<T>);

            head.as_ptr().write(Head { next: None, len: 0 });

            FatPtr { raw: (head, cap) }.fat
        }
    }

    fn dealloc(ptr: NonNull<Self>) {
        let cap = {
            let page = Self::as_mut(ptr);

            for item in page.data[..page.len].iter_mut() {
                unsafe { item.assume_init_drop() };
            }

            page.capacity()
        };

        unsafe { dealloc(ptr.as_ptr().cast(), Self::layout(cap)) }
    }

    fn layout(cap: usize) -> Layout {
        Layout::new::<Head<T>>()
            .extend(Layout::array::<T>(cap).unwrap())
            .unwrap()
            .0
            .pad_to_align()
    }

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }
}

impl<T> Deref for Page<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.get_ptr(), self.len) }
    }
}

impl<T> DerefMut for Page<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.get_mut_ptr(), self.len) }
    }
}

impl<T> Page<T> {
    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn get_ptr(&self) -> *const T {
        &self.data as *const _ as *const T
    }

    fn get_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    fn next_slot(&mut self) -> Option<&mut MaybeUninit<T>> {
        match self.data.get_mut(self.len) {
            Some(slot) => {
                self.len += 1;

                Some(slot)
            }
            None => None,
        }
    }
}

struct PageList<T> {
    page: NonNull<Page<T>>,
}

impl<T> PageList<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let page = Page::new(capacity);

        PageList { page }
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            fold: 0,
            page: self.page,
            _pl: PhantomData,
        }
    }
}

impl<T> Drop for PageList<T> {
    fn drop(&mut self) {
        let mut current = self.page;
        let mut next;

        loop {
            next = Page::<T>::as_mut(current).next;
            Page::<T>::dealloc(current);

            match next {
                Some(page) => current = page,
                None => break,
            }
        }
    }
}

struct Cursor<'a, T> {
    fold: usize,
    page: NonNull<Page<T>>,
    _pl: PhantomData<&'a mut Page<T>>,
}

impl<'a, T> Cursor<'a, T> {
    /// Drop all items and pages after current cursor position
    pub fn truncate_rest(self) -> Tail<'a, T> {
        let page = Page::as_mut(self.page);

        if let Some(to_drop) = page.data.get_mut(self.fold..page.len) {
            for item in to_drop {
                unsafe { item.assume_init_drop() };
                page.len = self.fold;
            }
        }

        if let Some(next) = page.next.take() {
            Page::dealloc(next);
        }

        Tail {
            page: self.page,
            _pl: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Cursor<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let page = Page::as_mut(self.page);

        if self.fold < page.len {
            let item = unsafe { page.data.get_unchecked_mut(self.fold).assume_init_mut() };

            self.fold += 1;

            return Some(item);
        }

        self.page = page.next?;
        self.fold = 0;

        self.next()
    }
}

struct Tail<'a, T> {
    page: NonNull<Page<T>>,
    _pl: PhantomData<&'a mut Page<T>>,
}

impl<T> Tail<'_, T> {
    pub fn extend<V, I, F>(&mut self, iter: I, f: F)
    where
        F: Fn(In<T>, V) -> Out<T>,
        I: IntoIterator<Item = V>,
    {
        let mut iter = iter.into_iter();
        let mut page = Page::as_mut(self.page);

        while let Some(item) = iter.next() {
            if let Some(slot) = page.next_slot() {
                unsafe { In::pinned(Pin::new_unchecked(slot), |p| f(p, item)) };
                continue;
            }

            let new = Page::new(iter.size_hint().0 + 1);

            page.next = Some(new);
            page = Page::as_mut(new);

            unsafe { In::pinned(Pin::new_unchecked(&mut page.data[0]), |p| f(p, item)) };
            page.len = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page() {
        let page = Page::<u32>::new(2);

        {
            let page = Page::as_mut(page);

            assert_eq!(page.len(), 0);
            assert_eq!(page.capacity(), 2);

            assert_eq!(&page[..], &[]);

            assert_eq!(page.next_slot().map(|slot| slot.write(42)), Some(&mut 42));
            assert_eq!(page.next_slot().map(|slot| slot.write(100)), Some(&mut 100));
            assert!(page.next_slot().is_none());

            assert_eq!(&page[..], &[42, 100]);
        }

        Page::dealloc(page);
    }

    /// Run this test in miri to check if there are no leaks
    #[test]
    fn page_non_copy() {
        let page = Page::<String>::new(3);

        {
            let page = Page::as_mut(page);

            assert_eq!(page.len(), 0);
            assert_eq!(page.capacity(), 3);

            assert_eq!(&page[..], &[] as &[&str]);

            page.next_slot().unwrap().write("foo".to_string());
            page.next_slot().unwrap().write("bar".to_string());

            assert_eq!(&page[..], &["foo", "bar"]);
        }

        Page::dealloc(page);
    }

    #[test]
    fn page_list() {
        let mut list = PageList::<u32>::with_capacity(2);

        let mut tail = list.cursor().truncate_rest();

        tail.extend([42, 100, 404], |p, n| p.put(n));

        assert_eq!(&Page::as_mut(list.page)[..], [42, 100]);

        assert_eq!(
            &list.cursor().map(|i| *i).collect::<Vec<_>>()[..],
            &[42, 100, 404]
        );
    }

    #[test]
    fn cursor_truncate() {
        let mut list = PageList::<u32>::with_capacity(2);

        let mut tail = list.cursor().truncate_rest();

        tail.extend([42, 100, 404], |p, n| p.put(n));

        let mut cursor = list.cursor();

        assert_eq!(cursor.next(), Some(&mut 42));

        let mut tail = cursor.truncate_rest();

        tail.extend([7], |p, n| p.put(n));

        assert_eq!(&Page::as_mut(list.page)[..], [42, 7]);
        assert_eq!(Page::as_mut(list.page).next, None);
    }
}
