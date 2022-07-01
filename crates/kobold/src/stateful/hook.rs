use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::mem::ManuallyDrop;

use crate::stateful::{Inner, Context, ShouldRender};
use crate::Html;

struct HookVTable<S> {
    state: fn(*const ()) -> *const RefCell<S>,
    render: fn(*const ()),
    clone: fn(*const ()) -> *const (),
    drop: fn(*const ()),
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

                    &(*inner).state
                },
                render: |inner| unsafe {
                    let inner = inner as *const Inner<S, H::Product>;
                    let render = (*inner).render.cast::<H>();
                    let state = (*inner).state.borrow();
                    let ctx = Context::new(inner);

                    render(&state, ctx);
                },
                clone: |inner| {
                    let weak = ManuallyDrop::new(unsafe { Weak::from_raw(inner as *const Inner<S, H::Product>) });

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
    pub fn update(&self, mutator: impl FnOnce(&mut S) -> ShouldRender) {
        let state = (self.vtable.state)(self.inner);
        let result = unsafe { mutator(&mut (*state).borrow_mut()) };

        if result.should_render() {
            (self.vtable.render)(self.inner)
        }
    }
}

impl<S> Clone for Hook<S>
where
    S: 'static,
{
    fn clone(&self) -> Self {
        let inner = (self.vtable.clone)(self.inner);

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
        (self.vtable.drop)(self.inner);
    }
}
