use std::cell::UnsafeCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

use crate::stateful::Inner;
use crate::{Element, Html, Mountable, ShouldRender};

pub struct Link<S, P> {
    pub(super) inner: Weak<Inner<S, P>>,
}

impl<S, P> Link<S, P>
where
    S: 'static,
    P: 'static,
{
    pub fn callback<F, A>(&self, cb: F) -> Callback<F, &Self>
    where
        F: Fn(&mut S, &Event) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback { cb, link: self }
    }
}

impl<S, P> Clone for Link<S, P> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner.clone(),
        }
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
    inner: Rc<UnsafeCell<F>>,
}

impl<F, A, S, P> Html for Callback<F, &Link<S, P>>
where
    F: Fn(&mut S, &Event) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
    P: 'static,
{
    type Product = CallbackProduct<F>;

    fn build(self) -> Self::Product {
        let link = self.link.clone();
        let inner = Rc::new(UnsafeCell::new(self.cb));
        let weak = Rc::downgrade(&inner);

        let closure = make_closure(move |event| {
            if let Some((rc, cb)) = link.inner.upgrade().zip(weak.upgrade()) {
                let cb = unsafe { &*cb.get() };
                if cb(&mut rc.state.borrow_mut(), event).into().should_render() {
                    rc.update();
                }
            }
        });
        let closure = Closure::wrap(closure);

        CallbackProduct { closure, inner }
    }

    fn update(self, p: &mut Self::Product) {
        unsafe { *p.inner.get() = self.cb }
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
