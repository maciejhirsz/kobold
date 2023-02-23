use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[repr(transparent)]
pub struct Event<E, T> {
    event: web_sys::Event,
    _target: PhantomData<(E, T)>,
}

pub type UntypedEvent<T = HtmlElement> = Event<web_sys::Event, T>;

pub type MouseEvent<T = HtmlElement> = Event<web_sys::MouseEvent, T>;

pub type KeyboardEvent<T = HtmlElement> = Event<web_sys::KeyboardEvent, T>;

impl<E, T> Deref for Event<E, T>
where
    E: JsCast,
{
    type Target = E;

    fn deref(&self) -> &E {
        self.event.unchecked_ref()
    }
}

impl<E, T> Event<E, T> {
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
