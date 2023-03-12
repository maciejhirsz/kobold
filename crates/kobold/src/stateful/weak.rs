use std::cell::{BorrowError, BorrowMutError, RefCell};
use std::fmt::{self, Display};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

use crate::stateful::{Hook, Inner, ShouldRender};
use crate::Html;

/// Error type returned by [`WeakHook::update`](WeakHook::update) and [`WeakHook::with`](WeakHook::with).
#[derive(Debug)]
pub enum HookError {
    /// Returned if the state has already been dropped, happens if the attempted
    /// update is applied to a component that has been removed from view.
    StateDropped,

    /// Attempted update while the state is mutably borrowed for another update.
    CycleDetected,
}

impl From<BorrowMutError> for HookError {
    fn from(_: BorrowMutError) -> Self {
        HookError::CycleDetected
    }
}

impl From<BorrowError> for HookError {
    fn from(_: BorrowError) -> Self {
        HookError::CycleDetected
    }
}

impl Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HookError::StateDropped => f.write_str("Could not update state: State was dropped"),
            HookError::CycleDetected => {
                f.write_str("Cycle detected: Attempting to update state during an ongoing update")
            }
        }
    }
}

impl std::error::Error for HookError {}

struct HookVTable<S: 'static> {
    state: unsafe fn(*const ()) -> Option<*const RefCell<Hook<S>>>,
    rerender: unsafe fn(&Hook<S>, *const ()),
    clone: unsafe fn(*const ()) -> *const (),
    drop: unsafe fn(*const ()),
}

/// Similar to [`Hook`](crate::stateful::Hook), the `WeakHook` is a smart pointer to
/// some state `S` that allows mutation of said state, and which will trigger a re-render
/// of components using that state. Unlike `Hook` however, `WeakHook` is passed by value
/// and can be freely cloned.
///
/// As the name suggests, this is a _weak_ reference, meaning that the state can be dropped
/// while a `WeakHook` to it exists, at which point interacting with this `WeakHook` will
/// produce errors.
pub struct WeakHook<S: 'static> {
    inner: *const (),
    vtable: &'static HookVTable<S>,
}

impl<S> WeakHook<S>
where
    S: 'static,
{
    pub(super) fn new<H>(inner: &Rc<Inner<S, H::Product>>) -> Self
    where
        H: Html,
    {
        Self::from_weak(Rc::downgrade(inner))
    }

    pub(super) fn from_weak<P: 'static>(inner: Weak<Inner<S, P>>) -> Self {
        let inner = inner.into_raw() as *const ();

        WeakHook {
            inner,
            vtable: &HookVTable {
                state: |inner| unsafe {
                    let inner = inner as *const Inner<S, P>;
                    let weak = ManuallyDrop::new(Weak::from_raw(inner));

                    if weak.strong_count() > 0 {
                        Some(&(*inner).hook)
                    } else {
                        None
                    }
                },
                rerender: |hook, inner| unsafe {
                    let inner = inner as *const Inner<S, P>;

                    (*inner).rerender(hook);
                },
                clone: |inner| {
                    let weak =
                        ManuallyDrop::new(unsafe { Weak::from_raw(inner as *const Inner<S, P>) });

                    Weak::into_raw((*weak).clone()) as *const ()
                },
                drop: |inner| unsafe {
                    Weak::from_raw(inner as *const Inner<S, P>);
                },
            },
        }
    }
}

impl<S> WeakHook<S>
where
    S: 'static,
{
    pub fn update<R>(&self, mutator: impl FnOnce(&mut S) -> R) -> Result<(), HookError>
    where
        R: Into<ShouldRender>,
    {
        let state = unsafe { (self.vtable.state)(self.inner) }.ok_or(HookError::StateDropped)?;
        let mut hook = unsafe { (*state).try_borrow_mut()? };
        let result = mutator(&mut hook.state);

        if result.into().should_render() {
            unsafe { (self.vtable.rerender)(&hook, self.inner) }
        }

        Ok(())
    }

    pub fn with<R>(&self, getter: impl FnOnce(&S) -> R) -> Result<R, HookError>
    where
        R: 'static,
    {
        let state = unsafe { (self.vtable.state)(self.inner) }.ok_or(HookError::StateDropped)?;
        let hook = unsafe { (*state).try_borrow()? };

        Ok(getter(&hook.state))
    }
}

impl<S> Clone for WeakHook<S>
where
    S: 'static,
{
    fn clone(&self) -> Self {
        let inner = unsafe { (self.vtable.clone)(self.inner) };

        WeakHook {
            inner,
            vtable: self.vtable,
        }
    }
}

impl<S> Drop for WeakHook<S>
where
    S: 'static,
{
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.inner) }
    }
}
