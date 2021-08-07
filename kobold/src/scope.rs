// use crate::traits::{Component, HandleMessage};
// use crate::traits::{Html, Update};
use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
// use web_sys::Event;

#[derive(Clone, Copy)]
enum Guard {
    /// Scope contains uninitialized data, this should only
    /// be set inside `UninitContext<T>`.
    Uninit,
    /// Data is initialized and there are no active borrows to it.
    Ready,
    /// Data is initialized and there is an active borrow to it.
    Borrowed,
    /// Data is initialized, there _might_ be an active borrow to it,
    /// but it's been requested to drop.
    DropRequested,
}

struct ScopeInner<T> {
    data: UnsafeCell<T>,
    guard: Cell<Guard>,
    links: Cell<u32>,
}

#[repr(transparent)]
pub struct Link<T> {
    ptr: NonNull<ScopeInner<T>>,
}

impl<T: Debug> Debug for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Link")
    }
}

#[repr(transparent)]
pub(crate) struct Scope<T> {
    ptr: NonNull<ScopeInner<T>>,
}

#[repr(transparent)]
pub(crate) struct UninitContext<T> {
    ptr: NonNull<ScopeInner<MaybeUninit<T>>>,
}

pub struct Ref<'a, T>(&'a ScopeInner<T>);

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { 
            &*self.0.data.get()
        }
    }
}

impl<T> DerefMut for Ref<'_, T> {
    // type Target = T
    fn deref_mut(&mut self) -> &mut T {
        unsafe { 
            &mut *self.0.data.get()
        }
    }
}

impl<T> Scope<T> {
    pub fn new_uninit() -> UninitContext<T> {
        use std::alloc::{alloc, Layout};

        let ptr = unsafe {
            let ptr = alloc(Layout::new::<ScopeInner<MaybeUninit<T>>>()) as *mut ScopeInner<MaybeUninit<T>>;

            (*ptr).links = Cell::new(0);
            (*ptr).guard = Cell::new(Guard::Uninit);

            ptr
        };

        UninitContext {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        let inner = unsafe { self.ptr.as_ref() };

        debug_assert!(matches!(&inner.guard.get(), Guard::Ready));

        match inner.guard.get() {
            Guard::Ready => {
                inner.guard.set(Guard::Borrowed);
                Ref(inner)
            },
            _ => panic!(),
        }
    }
}

impl<T> UninitContext<T> {
    pub fn make_link(&self) -> Link<T> {
        let inner = unsafe { self.ptr.as_ref() };

        inner.links.set(inner.links.get() + 1);

        Link { ptr: self.ptr.cast() }
    }

    pub fn init(mut self, data: T) -> Scope<T> {
        let inner = unsafe { self.ptr.as_mut() };

        debug_assert!(matches!(inner.guard.get(), Guard::Uninit));

        {
            let data_ptr = inner.data.get();

            unsafe { *data_ptr = MaybeUninit::new(data); }
        }
        inner.guard.set(Guard::Ready);

        Scope {
            ptr: self.ptr.cast(),
        }
    }
}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };

        let count = inner.links.get();
        inner.links.set(count + 1);

        Link { ptr: self.ptr }
    }
}

impl<T> Drop for Link<T> {
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

impl<T> Drop for Scope<T> {
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

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        debug_assert!(matches!(self.0.guard.get(), Guard::Borrowed));

        self.0.guard.set(Guard::Ready);
    }
}

impl<T> Link<T> {
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

// impl<T> Link<T> {
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
