use std::ops::Deref;

use wasm_bindgen::JsCast;

#[repr(transparent)]
pub struct Event {
    event: web_sys::Event,
}

impl Deref for Event {
    type Target = web_sys::Event;

    fn deref(&self) -> &web_sys::Event {
        &self.event
    }
}

impl Event {
    pub fn target<T: JsCast>(&self) -> T {
        self.event.target().unwrap().unchecked_into()
    }
}

impl<'a> From<&'a web_sys::Event> for &'a Event {
    fn from(event: &'a web_sys::Event) -> &'a Event {
        unsafe { &*(event as *const _ as *const Event) }
    }
}

impl<'a> From<&'a Event> for &'a web_sys::Event {
    fn from(event: &'a Event) -> &'a web_sys::Event {
        unsafe { &*(event as *const _ as *const Event) }
    }
}
