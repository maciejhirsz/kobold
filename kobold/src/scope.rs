// use crate::traits::{Component, HandleMessage};
use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
// use web_sys::Event;

#[derive(Clone, Copy)]
enum Guard {
    Ready,
    Borrowed,
    Dropped,
}

enum Guarded<T> {
    Uninit,
    Data(Data<T>),
    Dropped,
}

struct Data<T> {
    guard: Cell<Guard>,
    data: UnsafeCell<T>,
}

struct ContextInner<T> {
    links: Cell<usize>,
    guarded: Guarded<T>,
}

#[repr(transparent)]
pub struct Link<T> {
    ptr: NonNull<ContextInner<T>>,
}

#[repr(transparent)]
pub(crate) struct Scope<T> {
    ptr: NonNull<ContextInner<T>>,
}

#[repr(transparent)]
pub(crate) struct UninitContext<T> {
    ptr: NonNull<ContextInner<T>>,
}

pub struct Ref<'a, T>(&'a Data<T>);

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for Ref<'_, T> {
    // type Target = T
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}

impl<T> Scope<T> {
    pub fn new_uninit() -> UninitContext<T> {
        let ptr = Box::into_raw(Box::new(ContextInner {
            links: Cell::new(0),
            guarded: Guarded::Uninit,
        }));

        UninitContext {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        let inner = unsafe { self.ptr.as_ref() };

        debug_assert!(matches!(&inner.guarded, Guarded::Data(_)));

        match &inner.guarded {
            Guarded::Data(data) => match data.guard.get() {
                Guard::Ready => {
                    data.guard.set(Guard::Borrowed);
                    Ref(data)
                }
                _ => todo!("Error handling"),
            },
            _ => unsafe {
                // `Scope` can only be construted with `Init` variant,
                // and it only changes to `Dropped` when `Scope` is dropped.
                std::hint::unreachable_unchecked();
            },
        }
    }
}

impl<T> UninitContext<T> {
    pub fn make_link(&self) -> Link<T> {
        let inner = unsafe { self.ptr.as_ref() };

        inner.links.set(inner.links.get() + 1);

        Link { ptr: self.ptr }
    }

    pub fn init(mut self, data: T) -> Scope<T> {
        let inner = unsafe { self.ptr.as_mut() };

        inner.guarded = Guarded::Data(Data {
            guard: Cell::new(Guard::Ready),
            data: UnsafeCell::new(data),
        });

        Scope { ptr: self.ptr }
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

        let count = inner.links.get();

        match (count, &inner.guarded) {
            // This link is now the unique pointer
            (1, Guarded::Dropped) => {
                drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
            }
            _ => {
                inner.links.set(count - 1);
            }
        }
    }
}

impl<T> Drop for Scope<T> {
    fn drop(&mut self) {
        let count = unsafe { self.ptr.as_ref().links.get() };

        if count == 0 {
            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        } else {
            unsafe {
                self.ptr.as_mut().guarded = Guarded::Dropped;
            }
        }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        debug_assert!(matches!(self.0.guard.get(), Guard::Borrowed));

        self.0.guard.set(Guard::Ready);
    }
}

// impl<T> Link<T> {
//     fn new() -> Self {
//         Box::into_raw(Box::new(ContextInner {
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
