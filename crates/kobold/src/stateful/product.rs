// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use wasm_bindgen::JsValue;

use crate::internal::{In, Out};
use crate::stateful::Hook;
use crate::{init, Mountable, View};

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
    pub fn build<V>(updater: F, view: V, p: In<Self>) -> Out<Self>
    where
        V: View<Product = P>,
    {
        p.in_place(|p| unsafe {
            init!(p.updater = updater);
            init!(p.product @ view.build(p));

            Out::from_raw(p)
        })
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
