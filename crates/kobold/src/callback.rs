use crate::traits::{Html, Mountable, Update};
use std::cell::UnsafeCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

pub struct Callback<F>(pub F);

pub struct BoundCallback<F> {
    fun: Rc<UnsafeCell<F>>,
    closure: Closure<dyn FnMut(&Event)>,
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

impl<F> Html for Callback<F>
where
    F: FnMut(&Event) + 'static,
{
    type Built = BoundCallback<F>;

    fn build(self) -> Self::Built {
        let fun = Rc::new(UnsafeCell::new(self.0));
        let inner = fun.clone();

        let closure = make_closure(move |event| unsafe { (*inner.get())(event) });
        let closure = Closure::wrap(closure);

        BoundCallback { fun, closure }
    }
}

impl<F> Update<Callback<F>> for BoundCallback<F> {
    fn update(&mut self, new: Callback<F>) {
        unsafe { *self.fun.get() = new.0 };
    }
}

impl<F> Mountable for BoundCallback<F> {
    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}
