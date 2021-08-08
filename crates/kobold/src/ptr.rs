use crate::traits::MessageHandler;
use std::cell::{Cell, UnsafeCell};
use std::fmt::{self, Debug};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
// use web_sys::Event;

#[derive(Clone, Copy, Debug)]
enum Guard {
    /// Data is uninitialized and mustn't be borrowed.
    Uninit,
    /// Data is initialized and there are no active borrows to it.
    Ready,
    /// Data is initialized and there is an active borrow to it.
    Borrowed,
    /// Data is initialized, there _might_ be an active borrow to it,
    /// but it's been requested to drop.
    DropRequested,
}

/// Layout for the data held by `Prime` and `Weak` pointers.
struct Inner<T: ?Sized> {
    /// Guard to the data.
    guard: Cell<Guard>,
    /// Number of `Weak` references that have access to this data.
    refs: Cell<u32>,
    /// User defined data that can be `borrow`ed out of a `Prime` or `Weak` reference.
    data: UnsafeCell<T>,
}

/// `Prime` pointer, this is roughly equivalent to `Rc<RefCell<T>>`, except
/// there can only ever be one strong reference to it (hence the name).
#[repr(transparent)]
pub(crate) struct Prime<T: ?Sized> {
    ptr: NonNull<Inner<T>>,
}

/// `Weak` reference to some data owned by the `Prime` pointer. If the `Prime`
/// pointer is dropped, it will no longer be possible to borrow data from any
/// `Weak` reference created from it.
#[repr(transparent)]
pub struct Weak<T: ?Sized> {
    ptr: NonNull<Inner<T>>,
}

impl<T: Debug + ?Sized> Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Weak")
    }
}

pub struct Ref<'a, T: ?Sized>(&'a Inner<T>);

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T: ?Sized> DerefMut for Ref<'_, T> {
    // type Target = T
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}

/// Helper for alloc to make sure the pointer to `T` is using the same
/// `T` as the `Layout` that was used to make the allocation.
unsafe fn alloc<T>() -> *mut T {
    use std::alloc::{alloc, Layout};

    alloc(Layout::new::<T>()) as *mut T
}

impl<T> Prime<T> {
    pub fn new_uninit() -> Prime<T> {
        let ptr = unsafe {
            let ptr = alloc::<Inner<T>>();

            (*ptr).refs = Cell::new(0);
            (*ptr).guard = Cell::new(Guard::Uninit);

            ptr
        };

        Prime {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn init(&mut self, data: T) {
        let inner = unsafe { self.ptr.as_mut() };

        debug_assert!(matches!(inner.guard.get(), Guard::Uninit));

        {
            let data_ptr = inner.data.get();

            unsafe {
                data_ptr.write(data);
            }
        }
        inner.guard.set(Guard::Ready);
    }
}

impl<T: ?Sized> Prime<T> {
    pub fn borrow(&self) -> Option<Ref<T>> {
        let inner = unsafe { self.ptr.as_ref() };

        debug_assert!(!matches!(&inner.guard.get(), Guard::Uninit));

        match inner.guard.get() {
            Guard::Ready => {
                inner.guard.set(Guard::Borrowed);
                Some(Ref(inner))
            }
            _ => None,
        }
    }

    pub fn new_weak(&self) -> Weak<T> {
        let inner = unsafe { self.ptr.as_ref() };

        inner.refs.set(inner.refs.get() + 1);

        Weak { ptr: self.ptr }
    }
}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };

        let count = inner.refs.get();
        inner.refs.set(count + 1);

        Weak { ptr: self.ptr }
    }
}

impl<T> Weak<T> {
    /// Make generic when CoerceUnized is stabilized: https://github.com/rust-lang/rust/issues/27732
    pub(crate) fn coerce<M>(self) -> Weak<dyn MessageHandler<Message = M>>
    where
        T: MessageHandler<Message = M> + 'static,
    {
        let this = ManuallyDrop::new(self);

        Weak { ptr: this.ptr }
    }
}

impl<T: ?Sized> Drop for Weak<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };

        match (inner.refs.get(), inner.guard.get()) {
            // This link is now the unique pointer
            (1, Guard::DropRequested) => {
                drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
            }
            (n, _) => {
                inner.refs.set(n - 1);
            }
        }
    }
}

impl<T: ?Sized> Drop for Prime<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };

        if inner.refs.get() == 0 {
            debug_assert!(matches!(inner.guard.get(), Guard::Ready));

            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        } else {
            inner.guard.set(Guard::DropRequested);
        }
    }
}

impl<T: ?Sized> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        debug_assert!(matches!(self.0.guard.get(), Guard::Borrowed));

        self.0.guard.set(Guard::Ready);
    }
}

impl<T: ?Sized> Weak<T> {
    pub fn borrow(&self) -> Option<Ref<T>> {
        let inner = unsafe { self.ptr.as_ref() };

        debug_assert!(!matches!(inner.guard.get(), Guard::Uninit));

        match inner.guard.get() {
            Guard::Ready => {
                inner.guard.set(Guard::Borrowed);
                Some(Ref(inner))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_init() {
        let mut prime = Prime::new_uninit();

        prime.init(vec![42_u32]);

        assert_eq!(&**prime.borrow(), &[42]);

        drop(prime);
    }
}
