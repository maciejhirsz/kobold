// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for handling DOM events

use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, HtmlInputElement};

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
            pub struct $event<T> {
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

            impl<T> hidden::EventCast for $event<T> {}

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
                pub fn target(&self) -> EventTarget<T>
                where
                    T: JsCast,
                {
                    EventTarget(self.event.unchecked_ref::<EventWithTarget>().target().unchecked_into())
                }
            }
        )*
    };
}

mod hidden {
    pub trait EventCast {}
}

event! {
    /// [`web_sys::Event`](web_sys::Event)
    Event,
    /// [`web_sys::KeyboardEvent`](web_sys::KeyboardEvent)
    KeyboardEvent,
    /// [`web_sys::MouseEvent`](web_sys::MouseEvent)
    MouseEvent,
}

pub trait Listener<E>
where
    E: hidden::EventCast,
    Self: Sized + 'static,
{
    fn build(self) -> ListenerProduct<Self>;

    fn update(self, p: &mut ListenerProduct<Self>);
}

impl<E, F> Listener<E> for F
where
    F: FnMut(E) + 'static,
    E: hidden::EventCast,
{
    fn build(self) -> ListenerProduct<Self> {
        let raw = Box::into_raw(Box::new(self));

        let js = Closure::wrap(unsafe {
            Box::from_raw(raw as *mut dyn FnMut(E) as *mut dyn FnMut(web_sys::Event))
        })
        .into_js_value();

        // `into_js_value` will _forget_ the previous Box, so we can safely reconstruct it
        let boxed = unsafe { Box::from_raw(raw) };

        ListenerProduct { js, boxed }
    }

    fn update(self, p: &mut ListenerProduct<Self>) {
        *p.boxed = self;
    }
}

pub struct ListenerProduct<F> {
    js: JsValue,
    boxed: Box<F>,
}

impl<F> ListenerProduct<F> {
    pub fn js(&self) -> &JsValue {
        &self.js
    }
}

/// A wrapper over some event target type from web-sys.
#[repr(transparent)]
pub struct EventTarget<T>(T);

impl<T> Deref for EventTarget<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EventTarget<HtmlInputElement> {
    pub fn focus(&self) {
        drop(self.0.focus());
    }
}
