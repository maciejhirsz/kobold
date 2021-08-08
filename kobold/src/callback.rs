use crate::traits::{Html, Mountable, Update};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

pub struct Callback<F>(pub F);

pub struct BoundCallback<F> {
    // TODO: Have Link.bind closures use their own container, and make this
    // an Rc<RefCell<F>> again.
    fun: Box<F>,
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
        let mut fun = Box::new(self.0);
        let inner = &mut *fun as *mut F;

        let closure = make_closure(move |event| unsafe { (*inner)(event) });
        let closure = Closure::wrap(closure);

        BoundCallback { fun, closure }
    }
}

impl<F> Update<Callback<F>> for BoundCallback<F> {
    fn update(&mut self, new: Callback<F>) {
        *self.fun = new.0;
    }
}

impl<F> Mountable for BoundCallback<F> {
    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}
