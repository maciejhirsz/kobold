//! Utilities for handling DOM events

use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

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
