use std::cell::{BorrowMutError, RefCell};
use std::fmt::{self, Display};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

use crate::stateful::{Inner, ShouldRender};
use crate::Html;

pub enum UpdateError {
    StateDropped,
    AlreadyBorrowed,
}

impl From<BorrowMutError> for UpdateError {
    fn from(_: BorrowMutError) -> Self {
        UpdateError::AlreadyBorrowed
    }
}

impl Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::StateDropped => f.write_str("Could not update state: State was dropped"),
            UpdateError::AlreadyBorrowed => {
                f.write_str("Could not update state: State was already borrowed")
            }
        }
    }
}

struct HookVTable<S> {
    state: unsafe fn(*const ()) -> Option<*const RefCell<S>>,
    rerender: unsafe fn(&S, *const ()),
    clone: unsafe fn(*const ()) -> *const (),
    drop: unsafe fn(*const ()),
}

pub struct Hook<S: 'static> {
    inner: *const (),
    vtable: &'static HookVTable<S>,
}

impl<S> Hook<S>
where
    S: 'static,
{
    pub(super) fn new<H>(inner: &Rc<Inner<S, H::Product>>) -> Self
    where
        H: Html,
    {
        let inner = Rc::downgrade(inner).into_raw() as *const ();

        Hook {
            inner,
            vtable: &HookVTable {
                state: |inner| unsafe {
                    let inner = inner as *const Inner<S, H::Product>;
                    let weak = ManuallyDrop::new(Weak::from_raw(inner));

                    if weak.strong_count() > 0 {
                        Some(&(*inner).state)
                    } else {
                        None
                    }
                },
                rerender: |state, inner| unsafe {
                    let inner = inner as *const Inner<S, H::Product>;

                    (*inner).rerender(state);
                },
                clone: |inner| {
                    let weak = ManuallyDrop::new(unsafe {
                        Weak::from_raw(inner as *const Inner<S, H::Product>)
                    });

                    Weak::into_raw((*weak).clone()) as *const ()
                },
                drop: |inner| unsafe {
                    Weak::from_raw(inner as *const Inner<S, H::Product>);
                },
            },
        }
    }
}

impl<S> Hook<S>
where
    S: 'static,
{
    pub fn update(&self, mutator: impl FnOnce(&mut S) -> ShouldRender) -> Result<(), UpdateError> {
        let state = unsafe { (self.vtable.state)(self.inner) }.ok_or(UpdateError::StateDropped)?;
        let mut state = unsafe { (*state).try_borrow_mut()? };
        let result = mutator(&mut state);

        if result.should_render() {
            unsafe { (self.vtable.rerender)(&state, self.inner) }
        }

        Ok(())
    }
}

impl<S> Clone for Hook<S>
where
    S: 'static,
{
    fn clone(&self) -> Self {
        let inner = unsafe { (self.vtable.clone)(self.inner) };

        Hook {
            inner,
            vtable: self.vtable,
        }
    }
}

impl<S> Drop for Hook<S>
where
    S: 'static,
{
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.inner) }
    }
}
