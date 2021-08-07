// use crate::traits::{Component, HandleMessage};
// use crate::traits::{Html, Update};
use crate::traits::{Component, MessageHandler};
use std::cell::{Cell, UnsafeCell};
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
// use web_sys::Event;

#[derive(Clone, Copy)]
enum Guard {
    /// Scope contains uninitialized data, this should only
    /// be set inside `UninitScope<T>`.
    Uninit,
    /// Data is initialized and there are no active borrows to it.
    Ready,
    /// Data is initialized and there is an active borrow to it.
    Borrowed,
    /// Data is initialized, there _might_ be an active borrow to it,
    /// but it's been requested to drop.
    DropRequested,
}

struct ScopeInner<T: ?Sized> {
    guard: Cell<Guard>,
    links: Cell<u32>,
    data: UnsafeCell<T>,
}

#[repr(transparent)]
pub struct Weak<T: ?Sized> {
    ptr: NonNull<ScopeInner<T>>,
}

impl<T: Debug + ?Sized> Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Weak")
    }
}

#[repr(transparent)]
pub(crate) struct Scope<T: ?Sized> {
    ptr: NonNull<ScopeInner<T>>,
}

pub struct Ref<'a, T: ?Sized>(&'a ScopeInner<T>);

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

impl<T> Scope<T> {
    pub fn new_uninit() -> Scope<T> {
        let ptr = unsafe {
            let ptr = alloc::<ScopeInner<T>>();

            (*ptr).links = Cell::new(0);
            (*ptr).guard = Cell::new(Guard::Uninit);

            ptr
        };

        Scope {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn init(mut self, data: T) -> Scope<T> {
        let inner = unsafe { self.ptr.as_mut() };

        debug_assert!(matches!(inner.guard.get(), Guard::Uninit));

        {
            let data_ptr = inner.data.get();

            unsafe {
                data_ptr.write(data);
            }
        }
        inner.guard.set(Guard::Ready);

        Scope { ptr: self.ptr }
    }
}

impl<T: ?Sized> Scope<T> {
    pub fn borrow(&self) -> Ref<T> {
        let inner = unsafe { self.ptr.as_ref() };

        debug_assert!(matches!(&inner.guard.get(), Guard::Ready));

        match inner.guard.get() {
            Guard::Ready => {
                inner.guard.set(Guard::Borrowed);
                Ref(inner)
            }
            _ => panic!(),
        }
    }

    pub fn new_weak(&self) -> Weak<T> {
        let inner = unsafe { self.ptr.as_ref() };

        inner.links.set(inner.links.get() + 1);

        Weak { ptr: self.ptr }
    }

    pub fn as_weak(&self) -> &Weak<T> {
        unsafe { &*(self as *const Scope<T> as *const Weak<T>) }
    }
}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };

        let count = inner.links.get();
        inner.links.set(count + 1);

        Weak { ptr: self.ptr }
    }
}

impl<T> Weak<T> {
    /// Make generic when CoerceUnized is stabilized: https://github.com/rust-lang/rust/issues/27732
    pub(crate) fn coerce<C: Component>(self) -> Weak<dyn MessageHandler<Component = C>>
    where
        T: MessageHandler<Component = C> + 'static,
    {
        Weak { ptr: self.ptr }
    }
}

impl<T: ?Sized> Drop for Weak<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };

        match (inner.links.get(), inner.guard.get()) {
            // This link is now the unique pointer
            (1, Guard::DropRequested) => {
                drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
            }
            (n, _) => {
                inner.links.set(n - 1);
            }
        }
    }
}

impl<T: ?Sized> Drop for Scope<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };

        if inner.links.get() == 0 {
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

// impl<T> Weak<T> {
//     fn new() -> Self {
//         Box::into_raw(Box::new(ScopeInner {
//             links: Cell::new(0),
//             data: Guarded::Uninit,
//         }))
//     }
// }

// pub struct Scope<Component> {
//     component: Rc<RefCell<Option<Component>>>,
// }

// impl<Component> Clone for Scope<Component> {
//     fn clone(&self) -> Self {
//         Scope {
//             component: self.component.clone(),
//         }
//     }
// }

// impl<Comp: Component> Scope<Comp> {
//     pub(crate) fn new(component: Comp) -> Self {
//         let component = Rc::new(RefCell::new(Some(component)));

//         Scope { component }
//     }

//     pub fn with(&self, f: impl FnOnce(&mut Comp)) {
//         if let Some(comp) = &mut *self.component.borrow_mut() {
//             f(comp);
//         }
//     }

//     pub fn bind<Callback, Message>(&self, callback: Callback) -> impl FnMut(&Event)
//     where
//         Callback: Fn(&Event) -> Message,
//         Comp: HandleMessage<Message>,
//     {
//         let weak = Rc::downgrade(&self.component);

//         move |event| {
//             let msg = callback(event);

//             if let Some(rc) = Weak::upgrade(&weak) {
//                 if let Ok(Some(comp)) = &mut rc.try_borrow_mut() {
//                     component.handle(msg);
//                 }
//             }
//         }
//     }
// }
