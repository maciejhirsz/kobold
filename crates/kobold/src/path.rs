// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use wasm_bindgen::JsValue;

use crate::internal::get_path;
use crate::runtime::POPSTATE_EID;
use crate::runtime::{EventContext, Then, Trigger};
use crate::{Mountable, View};

pub struct PopState<F> {
    render: F,
}

pub struct PopStateProduct<P> {
    path: String,
    product: P,
}

pub fn popstate<F, V>(render: F) -> PopState<F>
where
    F: Fn(&str) -> V,
    V: View,
{
    PopState { render }
}

impl<F, V> View for PopState<F>
where
    F: Fn(&str) -> V,
    V: View,
{
    type Product = PopStateProduct<V::Product>;

    fn build(self) -> Self::Product {
        let path = get_path();
        let view = (self.render)(&path);

        crate::internal::handle_pop_state();

        PopStateProduct {
            path,
            product: view.build(),
        }
    }

    fn update(self, p: &mut Self::Product) {
        let view = (self.render)(&p.path);

        view.update(&mut p.product);
    }
}

impl<P> Mountable for PopStateProduct<P>
where
    P: Mountable,
{
    type Js = P::Js;

    fn js(&self) -> &JsValue {
        self.product.js()
    }

    fn unmount(&self) {
        self.product.unmount()
    }

    fn replace_with(&self, new: &JsValue) {
        self.product.replace_with(new);
    }
}

impl<P> Trigger for PopStateProduct<P>
where
    P: Trigger,
{
    fn trigger<C: EventContext>(&mut self, ctx: &mut C) -> Option<Then> {
        if let Some(_) = ctx.event::<()>(POPSTATE_EID) {
            let path = get_path();

            if path != self.path {
                self.path = path;
                Some(Then::Render)
            } else {
                Some(Then::Stop)
            }
        } else {
            self.product.trigger(ctx)
        }
    }
}
