// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for handling DOM events

use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

use crate::{Mountable, View};

#[wasm_bindgen]
extern "C" {
    type EventWithTarget;

    #[wasm_bindgen(method, getter)]
    fn target(this: &EventWithTarget) -> HtmlElement;
}

macro_rules! event {
    ($(#[doc = $doc:literal] $event:ident,)*) => {
        $(
            #[doc = concat!("Smart wrapper around a ", $doc, "which includes the type information of the event target")]
            #[repr(transparent)]
            pub struct $event<T = HtmlElement> {
                event: web_sys::$event,
                _target: PhantomData<T>,
            }

            impl<T> From<web_sys::Event> for $event<T> {
                fn from(event: web_sys::Event) -> Self {
                    $event {
                        event: event.unchecked_into(),
                        _target: PhantomData,
                    }
                }
            }

            impl<T> Deref for $event<T> {
                type Target = web_sys::$event;

                fn deref(&self) -> &Self::Target {
                    &self.event.unchecked_ref()
                }
            }

            impl<T> $event<T> {
                /// Return a reference to the target element.
                ///
                /// This method shadows over the [`Event::target`](web_sys::Event::target)
                /// method provided by `web-sys` and makes it infallible.
                pub fn target(&self) -> T
                where
                    T: JsCast,
                {
                    self.event.unchecked_ref::<EventWithTarget>().target().unchecked_into()
                }
            }
        )*
    };
}

event! {
    /// [`web_sys::Event`](web_sys::Event)
    Event,
    /// [`web_sys::KeyboardEvent`](web_sys::KeyboardEvent)
    KeyboardEvent,
    /// [`web_sys::MouseEvent`](web_sys::MouseEvent)
    MouseEvent,
}

pub fn event_handler<E>(
    handler: impl Fn(E) + 'static,
) -> EventHandler<impl Fn(web_sys::Event) + 'static>
where
    E: From<web_sys::Event>,
{
    EventHandler(move |event| handler(E::from(event)))
}

pub struct EventHandler<F>(F);

pub struct ClosureProduct<F> {
    js: JsValue,
    boxed: Box<F>,
}

impl<F> ClosureProduct<F>
where
    F: FnMut(web_sys::Event) + 'static,
{
    fn make(f: F) -> Self {
        let raw = Box::into_raw(Box::new(f));

        let js = Closure::wrap(unsafe { Box::from_raw(raw) } as Box<dyn FnMut(web_sys::Event)>)
            .into_js_value();

        // `into_js_value` will _forget_ the previous Box, so we can safely reconstruct it
        let boxed = unsafe { Box::from_raw(raw) };

        ClosureProduct { js, boxed }
    }

    fn update(&mut self, f: F) {
        *self.boxed = f;
    }
}

impl<F> View for EventHandler<F>
where
    F: Fn(web_sys::Event) + 'static,
{
    type Product = ClosureProduct<F>;

    fn build(self) -> Self::Product {
        ClosureProduct::make(self.0)
    }

    fn update(self, p: &mut Self::Product) {
        p.update(self.0)
    }
}

impl<F> Mountable for ClosureProduct<F>
where
    F: 'static,
{
    type Js = JsValue;

    fn js(&self) -> &JsValue {
        &self.js
    }

    fn unmount(&self) {}

    fn replace_with(&self, _: &JsValue) {
        debug_assert!(false, "Using JsClosure as a DOM Node");
    }
}
