use std::marker::PhantomData;

use wasm_bindgen::JsValue;

use crate::stateful::Hook;
use crate::Mountable;

pub trait Product<S> {
    fn update(&mut self, hook: &Hook<S>);

    fn js(&self) -> &JsValue;

    fn unmount(&self);

    fn replace_with(&self, new: &JsValue);
}

pub struct ProductHandler<S, P, F> {
    updater: F,
    product: P,
    _state: PhantomData<S>,
}

impl<S, P, F> ProductHandler<S, P, F> {
    pub const fn new(updater: F, product: P) -> Self {
        ProductHandler {
            updater,
            product,
            _state: PhantomData,
        }
    }
}

impl<S, P, F> Product<S> for ProductHandler<S, P, F>
where
    S: 'static,
    P: Mountable,
    F: FnMut(*const Hook<S>, *mut P),
{
    fn update(&mut self, hook: &Hook<S>) {
        (self.updater)(hook, &mut self.product);
    }

    fn js(&self) -> &JsValue {
        self.product.js()
    }

    fn unmount(&self) {
        self.product.unmount()
    }

    fn replace_with(&self, new: &JsValue) {
        self.product.replace_with(new)
    }
}
