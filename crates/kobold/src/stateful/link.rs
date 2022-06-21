use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

use crate::stateful::Inner;
use crate::{Element, Html, Mountable, ShouldRender};

pub struct Link<'state, S> {
    inner: *const (),
    make_closure: fn(*const (), cb: Weak<UnsafeCell<dyn CallbackFn<S>>>) -> Box<dyn FnMut(&Event)>,
    _marker: PhantomData<&'state S>,
}

impl<S> Clone for Link<'_, S> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner,
            make_closure: self.make_closure,
            _marker: PhantomData,
        }
    }
}

impl<S> Copy for Link<'_, S> {}

impl<'state, S> Link<'state, S>
where
    S: 'static,
{
    pub(super) fn new<P: 'static>(inner: &'state Inner<S, P>) -> Self {
        Self::new_raw(inner)
    }

    pub(super) fn from_weak<P: 'static>(weak: &'state Weak<Inner<S, P>>) -> Self {
        Self::new_raw(weak.as_ptr())
    }

    fn new_raw<P: 'static>(inner: *const Inner<S, P>) -> Self {
        Link {
            inner: inner as *const _,
            make_closure: |inner, weak_cb| {
                make_closure(move |event| {
                    if let Some(cb) = weak_cb.upgrade() {
                        let inner = unsafe { &*(inner as *const Inner<S, P>) };
                        let cb = unsafe { &*cb.get() };

                        if cb
                            .call(&mut inner.state.borrow_mut(), event)
                            .should_render()
                        {
                            inner.update();
                        }
                    }
                })
            },
            _marker: PhantomData,
        }
    }

    pub(super) fn inner<P: 'static>(&self) -> *const Inner<S, P> {
        self.inner as *const Inner<S, P>
    }

    pub fn callback<F, A>(self, cb: F) -> Callback<F, Self>
    where
        F: Fn(&mut S, &Event) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback { cb, link: self }
    }
}

pub struct Callback<F, L> {
    cb: F,
    link: L,
}

// I should not need to write this, but lifetime checking
// was going really off the rails with inlined boxing
#[inline]
fn make_closure<F>(fun: F) -> Box<dyn FnMut(&Event)>
where
    F: FnMut(&Event) + 'static,
{
    Box::new(fun)
}

pub struct CallbackProduct<F> {
    closure: Closure<dyn FnMut(&Event)>,
    cb: Rc<UnsafeCell<F>>,
}

trait CallbackFn<S> {
    fn call(&self, state: &mut S, event: &Event) -> ShouldRender;
}

impl<F, A, S> CallbackFn<S> for F
where
    F: Fn(&mut S, &Event) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    fn call(&self, state: &mut S, event: &Event) -> ShouldRender {
        (self)(state, event).into()
    }
}

impl<F, A, S> Html for Callback<F, Link<'_, S>>
where
    F: Fn(&mut S, &Event) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    type Product = CallbackProduct<F>;

    fn build(self) -> Self::Product {
        let Self { link, cb } = self;

        let cb = Rc::new(UnsafeCell::new(cb));
        let weak = Rc::downgrade(&cb);

        let closure = (link.make_closure)(link.inner, weak);
        let closure = Closure::wrap(closure);

        CallbackProduct { closure, cb }
    }

    fn update(self, p: &mut Self::Product) {
        unsafe { *p.cb.get() = self.cb }
    }
}

impl<F: 'static> Mountable for CallbackProduct<F> {
    fn el(&self) -> &Element {
        panic!("Callback is not an element");
    }

    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}
