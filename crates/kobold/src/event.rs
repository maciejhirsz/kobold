use std::ops::Deref;
use std::marker::PhantomData;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub trait WithEventTarget<T> {}

#[repr(transparent)]
pub struct Event<T = HtmlElement> {
    event: web_sys::Event,
    _target: PhantomData<T>,
}

impl<T> Deref for Event<T> {
    type Target = web_sys::Event;

    fn deref(&self) -> &web_sys::Event {
        &self.event
    }
}

impl<T: JsCast> Event<T> {
    pub fn target(&self) -> T {
        self.event.target().unwrap().unchecked_into()
    }
}
