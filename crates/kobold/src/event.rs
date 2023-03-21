//! Utilities for handling DOM events

use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

use crate::{Element, Mountable, View};

/// Smart wrapper around a [`web_sys::Event`](web_sys::Event) which includes type
/// information for the target element of said event.
#[repr(transparent)]
pub struct Event<T = HtmlElement, E = web_sys::Event> {
    event: web_sys::Event,
    _target: PhantomData<(E, T)>,
}

pub type MouseEvent<T = HtmlElement> = Event<T, web_sys::MouseEvent>;

pub type KeyboardEvent<T = HtmlElement> = Event<T, web_sys::KeyboardEvent>;

impl<T, E> Deref for Event<T, E>
where
    E: JsCast,
{
    type Target = E;

    fn deref(&self) -> &E {
        self.event.unchecked_ref()
    }
}

impl<T, E> From<web_sys::Event> for Event<T, E> {
    fn from(event: web_sys::Event) -> Self {
        Event {
            event,
            _target: PhantomData,
        }
    }
}

impl<T, E> Event<T, E> {
    pub fn target(&self) -> T
    where
        T: JsCast,
    {
        self.event.target().unwrap().unchecked_into()
    }

    pub fn stop_propagation(&self) {
        self.event.stop_propagation();
    }

    pub fn stop_immediate_propagation(&self) {
        self.event.stop_immediate_propagation();
    }

    pub fn prevent_default(&self) {
        self.event.prevent_default();
    }
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

    fn el(&self) -> &Element {
        panic!("Closure is not an element");
    }

    fn js(&self) -> &JsValue {
        &self.js
    }
}
