use std::ops::Deref;
use std::marker::PhantomData;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

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

impl<'a, T> From<&'a web_sys::Event> for &'a Event<T> {
    fn from(event: &'a web_sys::Event) -> &'a Event<T> {
        unsafe { &*(event as *const _ as *const Event<T>) }
    }
}

impl<'a, T> From<&'a Event<T>> for &'a web_sys::Event {
    fn from(event: &'a Event<T>) -> &'a web_sys::Event {
        unsafe { &*(event as *const _ as *const Event) }
    }
}
