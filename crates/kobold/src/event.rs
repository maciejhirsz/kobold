// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for handling DOM events

use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, HtmlInputElement};

use crate::internal::{self, In, Out};

#[wasm_bindgen]
extern "C" {
    type EventWithTarget;

    #[wasm_bindgen(method, getter)]
    fn target(this: &EventWithTarget) -> HtmlElement;

    #[wasm_bindgen(method, getter, js_name = "currentTarget")]
    fn current_target(this: &EventWithTarget) -> HtmlElement;
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

            impl<T> EventCast for $event<T> {}

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
                pub fn target(&self) -> HtmlElement {
                    self.event.unchecked_ref::<EventWithTarget>().target().unchecked_into()
                }

                /// Return a reference to the target element.
                ///
                /// This method shadows over the [`Event::target`](web_sys::Event::target)
                /// method provided by `web-sys` and makes it infallible.
                pub fn current_target(&self) -> EventTarget<T>
                where
                    T: JsCast,
                {
                    EventTarget(self.event.unchecked_ref::<EventWithTarget>().current_target().unchecked_into())
                }
            }
        )*
    };
}

mod sealed {
    pub trait EventCast {}

    impl EventCast for web_sys::Event {}
}

pub(crate) use sealed::EventCast;

event! {
    /// [`web_sys::Event`](web_sys::Event)
    Event,
    /// [`web_sys::KeyboardEvent`](web_sys::KeyboardEvent)
    KeyboardEvent,
    /// [`web_sys::MouseEvent`](web_sys::MouseEvent)
    MouseEvent,
}

pub trait IntoListener<E: EventCast> {
    type Listener: Listener<E>;

    fn into_listener(self) -> Self::Listener;
}

impl<E, L> IntoListener<E> for L
where
    L: Listener<E>,
    E: EventCast,
{
    type Listener = L;

    fn into_listener(self) -> L {
        self
    }
}

pub trait Listener<E>
where
    E: EventCast,
    Self: Sized + 'static,
{
    type Product: ListenerHandle;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product>;

    fn update(self, p: &mut Self::Product);
}

impl<E, F> Listener<E> for F
where
    F: FnMut(E) + 'static,
    E: EventCast,
{
    type Product = ListenerProduct<Self, E>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        p.put(ListenerProduct {
            closure: self,
            _event: PhantomData,
        })
    }

    fn update(self, p: &mut ListenerProduct<Self, E>) {
        p.closure = self;
    }
}

pub struct ListenerProduct<F, E> {
    closure: F,
    _event: PhantomData<E>,
}

pub trait ListenerHandle {
    fn js_value(&mut self) -> JsValue;
}

impl<F, E> ListenerHandle for ListenerProduct<F, E>
where
    F: FnMut(E) + 'static,
    E: EventCast,
{
    fn js_value(&mut self) -> JsValue {
        let vcall: fn(E, *mut ()) = |e, ptr| unsafe { (*(ptr as *mut F))(e) };

        internal::make_event_handler((&mut self.closure) as *mut F as *mut (), vcall as usize)
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
