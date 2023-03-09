use std::cell::{BorrowMutError, BorrowError, RefCell};
use std::fmt::{self, Display};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

use crate::stateful::{Hook, Inner, ShouldRender};
use crate::Html;

/// Error type returned by [`Hook::update`](Hook::update).
#[derive(Debug)]
pub enum UpdateError {
    /// Returned if the state has already been dropped, happens if the attempted
    /// update is applied to a component that has been removed from view.
    StateDropped,

    /// Attempted update while the state is mutably borrowed for another update.
    CycleDetected,
}

impl From<BorrowMutError> for UpdateError {
    fn from(_: BorrowMutError) -> Self {
        UpdateError::CycleDetected
    }
}

impl From<BorrowError> for UpdateError {
    fn from(_: BorrowError) -> Self {
        UpdateError::CycleDetected
    }
}

impl Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::StateDropped => f.write_str("Could not update state: State was dropped"),
            UpdateError::CycleDetected => {
                f.write_str("Cycle detected: Attempting to update state during an ongoing update")
            }
        }
    }
}

impl std::error::Error for UpdateError {}

struct HookVTable<S> {
    state: unsafe fn(*const ()) -> Option<*const RefCell<Hook<S>>>,
    rerender: unsafe fn(&Hook<S>, *const ()),
    clone: unsafe fn(*const ()) -> *const (),
    drop: unsafe fn(*const ()),
}

pub struct OwnedHook<S: 'static> {
    inner: *const (),
    vtable: &'static HookVTable<S>,
}

impl<S> OwnedHook<S>
where
    S: 'static,
{
    pub(super) fn new<H>(inner: &Rc<Inner<S, H::Product>>) -> Self
    where
        H: Html,
    {
        let inner = Rc::downgrade(inner).into_raw() as *const ();

        OwnedHook {
            inner,
            vtable: &HookVTable {
                state: |inner| unsafe {
                    let inner = inner as *const Inner<S, H::Product>;
                    let weak = ManuallyDrop::new(Weak::from_raw(inner));

                    if weak.strong_count() > 0 {
                        Some(&(*inner).hook)
                    } else {
                        None
                    }
                },
                rerender: |hook, inner| unsafe {
                    let inner = inner as *const Inner<S, H::Product>;

                    (*inner).rerender(hook);
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

impl<S> OwnedHook<S>
where
    S: 'static,
{
    pub fn update<R>(&self, mutator: impl FnOnce(&mut S) -> R) -> Result<(), UpdateError>
    where
        R: Into<ShouldRender>,
    {
        let state = unsafe { (self.vtable.state)(self.inner) }.ok_or(UpdateError::StateDropped)?;
        let mut hook = unsafe { (*state).try_borrow_mut()? };
        let result = mutator(&mut hook.state);

        if result.into().should_render() {
            unsafe { (self.vtable.rerender)(&hook, self.inner) }
        }

        Ok(())
    }

    pub fn with<R: 'static>(&self, getter: impl FnOnce(&S) -> R) -> Result<R, UpdateError> {
        let state = unsafe { (self.vtable.state)(self.inner) }.ok_or(UpdateError::StateDropped)?;
        let hook = unsafe { (*state).try_borrow()? };

        Ok(getter(&hook.state))

    }
}

impl<S> Clone for OwnedHook<S>
where
    S: 'static,
{
    fn clone(&self) -> Self {
        let inner = unsafe { (self.vtable.clone)(self.inner) };

        OwnedHook {
            inner,
            vtable: self.vtable,
        }
    }
}

impl<S> Drop for OwnedHook<S>
where
    S: 'static,
{
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.inner) }
    }
}
