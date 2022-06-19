use std::rc::Weak;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

use crate::stateful::Inner;
use crate::{Html, Mountable, ShouldRender};

pub struct Link<S, P> {
    pub(in super) inner: Weak<Inner<S, P>>,
}

impl<S, P> Link<S, P>
where
    S: 'static,
    P: 'static,
{
    pub fn bind<F, A>(&self, cb: F) -> Callback<F, &Self>
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

pub struct CallbackProduct {
    closure: Closure<dyn FnMut(&Event)>,
}

impl<F, A, S, P> Html for Callback<F, &Link<S, P>>
where
    F: Fn(&mut S, &Event) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
    P: 'static,
{
    type Product = CallbackProduct;

    fn build(self) -> Self::Product {
        let link = self.link.clone();
        let cb = self.cb;

        let closure = make_closure(move |event| {
            if let Some(rc) = link.inner.upgrade() {
                if cb(&mut rc.state.borrow_mut(), event).into().should_render() {
                    rc.update();
                }
            }
        });
        let closure = Closure::wrap(closure);

        CallbackProduct { closure }
    }

    fn update(self, _: &mut Self::Product) {}
}

impl Mountable for CallbackProduct {
    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}
