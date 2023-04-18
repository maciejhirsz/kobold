// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use wasm_bindgen::JsValue;

use crate::internal::{Field, Mut, Pre};
use crate::stateful::Hook;
use crate::{init, Mountable, View};

pub trait Product<S> {
    fn update(&mut self, hook: &Hook<S>);

    fn js(&self) -> &JsValue;

    fn unmount(&self);

    fn replace_with(&self, new: &JsValue);
}

pub struct ProductHandler<S, P, F> {
    updater: Field<F>,
    product: P,
    _state: PhantomData<S>,
}

impl<S, P, F> ProductHandler<S, P, F> {
    pub fn new<V>(updater: F, view: V, p: Pre<Self>) -> Mut<Self>
    where
        V: View<Product = P>,
        P: Unpin,
        Self: Unpin,
    {
        unsafe {
            let p = p.into_raw();

            init!(p.updater = Field::new(updater));
            init!(p.product @ view.build(p));

            Mut::from_raw(p)
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
        (self.updater.get_mut())(hook, &mut self.product);
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
