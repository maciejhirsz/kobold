use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Function;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Event;

pub struct Callback<F> {
    fun: F,
}

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

impl<F> Callback<F>
where
    F: FnMut() + 'static,
{
    pub fn bind(self) -> BoundCallback<F> {
        let fun = Rc::new(RefCell::new(self.fun));
        let inner = fun.clone();

        let closure = make_closure(move |_event| inner.borrow_mut()());
        let closure = Closure::wrap(closure);

        BoundCallback { fun, closure }
    }
}

impl<F> BoundCallback<F> {
    pub fn update(&self, new: F) {
        self.fun.replace(new);
    }

    pub fn function(&self) -> &Function {
        self.closure.as_ref().unchecked_ref()
    }
}

impl<F> Callback<F> {
    pub fn new(fun: F) -> Self {
        Callback { fun }
    }
}
