use std::alloc::{alloc, dealloc, Layout};
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
    fn new(cap: usize) -> NonNull<Self> {
        // TODO: Use a dynamic MIN_PAGE_SIZE for T
        let cap = std::cmp::max(cap, 1);

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
    first: NonNull<Page<T>>,
    last: NonNull<Page<T>>,
}

impl<T> PageList<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = std::cmp::max(capacity, 1);
        let page = Page::new(capacity);

        PageList {
            first: page,
            last: page,
        }
    }

    pub fn push(&mut self, item: T) {
        let slot = match Page::as_mut(self.last).next_slot() {
            Some(slot) => slot,
            None => {
                let new = Page::new(Page::as_mut(self.last).capacity());
                let slot = Page::as_mut(new).next_slot().unwrap();

                Page::as_mut(self.last).next = Some(new);
                self.last = new;

                slot
            }
        };

        slot.write(item);
    }

    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            fold: 0,
            page: Page::as_mut(self.first),
        }
    }
}

impl<T> Drop for PageList<T> {
    fn drop(&mut self) {
        let mut current = self.first;
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
    page: &'a mut Page<T>,
}

impl<'a, T> Cursor<'a, T> {
    /// Drop all items and pages after current cursor position
    pub fn truncate_rest(&mut self) {
        if let Some(to_drop) = self.page.data.get_mut(self.fold..self.page.len) {
            for item in to_drop {
                unsafe { item.assume_init_drop() };
                self.page.len = self.fold;
            }
        }

        if let Some(next) = self.page.next.take() {
            Page::dealloc(next);
        }
    }

    pub fn reserve(mut self, cap: usize) -> Builder<'a, T> {
        while let Some(next) = self.page.next.map(Page::as_mut) {
            self.page = next;
        }

        if let Some(grow) = self.page.capacity().checked_sub(self.page.len + cap) {
            self.page.next = Some(Page::new(grow));
        }

        Builder { page: self.page }
    }
}

impl<'a, T> Iterator for Cursor<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        loop {
            if self.fold < self.page.len {
                let item = unsafe { self.page.data.get_unchecked_mut(self.fold) };

                self.fold += 1;

                return Some(unsafe { &mut *(item.as_mut_ptr() as *mut T) });
            }

            self.page = self.page.next.map(Page::as_mut)?;
            self.fold = 0;
        }
    }
}

struct Builder<'a, T> {
    page: &'a mut Page<T>,
}

impl<T> Builder<'_, T> {
    pub fn build<F>(&mut self, f: F)
    where
        F: FnOnce(In<T>) -> Out<T>,
    {
        let slot = loop {
            match self.page.data.get_mut(self.page.len) {
                Some(slot) => break slot,
                None => {
                    match self.page.next {
                        Some(next) => self.page = Page::as_mut(next),
                        None => return,
                    }
                },
            }
        };

        unsafe { In::pinned(Pin::new_unchecked(slot), f) };
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

        list.push(42);
        list.push(100);
        list.push(404);

        assert_eq!(&Page::as_mut(list.first)[..], [42, 100]);
        assert_eq!(&Page::as_mut(list.last)[..], [404]);

        assert_eq!(
            &list.cursor().map(|i| *i).collect::<Vec<_>>()[..],
            &[42, 100, 404]
        );
    }

    #[test]
    fn cursor_truncate() {
        let mut list = PageList::<u32>::with_capacity(2);

        list.push(42);
        list.push(100);
        list.push(404);

        let mut cursor = list.cursor();

        assert_eq!(cursor.next(), Some(&mut 42));

        cursor.truncate_rest();

        assert_eq!(cursor.next(), None);

        assert_eq!(&Page::as_mut(list.first)[..], [42]);
        assert_eq!(Page::as_mut(list.first).next, None);
    }
}
