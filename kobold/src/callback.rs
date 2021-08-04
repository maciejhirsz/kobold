use crate::traits::{Html, Mountable, Update};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

pub struct Callback<F>(pub F);

pub struct BoundCallback<F> {
    fun: Rc<RefCell<F>>,
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
    type Rendered = BoundCallback<F>;

    fn render(self) -> Self::Rendered {
        let fun = Rc::new(RefCell::new(self.0));
        let inner = fun.clone();

        let closure = make_closure(move |event| inner.borrow_mut()(event));
        let closure = Closure::wrap(closure);

        BoundCallback { fun, closure }
    }
}

impl<F> Update<Callback<F>> for BoundCallback<F> {
    fn update(&mut self, new: Callback<F>) {
        self.fun.replace(new.0);
    }
}

impl<F> Mountable for BoundCallback<F> {
    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}
