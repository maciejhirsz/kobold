use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;
use web_sys::Node;

mod util;

pub trait Html: Sized {
    type Built: Mountable;

    fn build(self) -> Self::Built;

    fn update(self, built: &mut Self::Built);
}

pub trait Mountable: 'static {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self, parent: &Node) {
        util::__kobold_unmount(parent, self.js());
    }
}

pub type ShouldRender = bool;

// pub trait EventedComponent: Sized {
//     type State: Render;

//     fn init(self, link: Link<Self::State>) -> Self::State;

//     fn update(self, link: Link<Self::State>, state: &mut Self::State) -> ShouldRender {
//         *state = self.init(link);

//         true
//     }
// }

pub trait Render: 'static {
    type Out: Html;

    fn render(&self) -> Self::Out;
}

// pub struct Link<'a, S: Render> {
//     state: &'a Weak<RefCell<S>>,
// }

// impl<S: Render> Link<'_, S> {
//     pub fn link(&self, f: impl Fn(&mut S) -> ShouldRender + 'static) -> Closure {
//         let state = self.state.clone();

//         Closure(Box::new(move || {
//             if let Some(rc) = state.upgrade() {
//                 let mut state = rc.borrow_mut();

//                 if f(&mut state) {
//                     state.render();
//                 }
//             }
//         }))
//     }
// }

pub struct Closure(Box<dyn FnMut()>);

trait Link<T> {
    type Hosted;

    fn host(item: T) -> Self::Hosted;
}

struct Pure;

impl<T> Link<T> for Pure {
    type Hosted = T;

    fn host(item: T) -> Self::Hosted {
        item
    }
}

#[repr(transparent)]
struct Evented<T>(Weak<RefCell<T>>);

impl<T: Render> Evented<T> {
    pub fn link(&self, f: impl Fn(&mut T) -> ShouldRender + 'static) -> Closure {
        let state = self.0.clone();

        Closure(Box::new(move || {
            if let Some(rc) = state.upgrade() {
                let mut state = rc.borrow_mut();

                if f(&mut state) {
                    state.render();
                }
            }
        }))
    }
}

impl<T> Link<T> for Evented<T> {
    type Hosted = Rc<RefCell<T>>;

    fn host(item: T) -> Self::Hosted {
        Rc::new(RefCell::new(item))
    }
}

trait Component: Sized {
    type Built: Mountable;

    type Link: Link<Self>;

    fn create(self, _link: &Self::Link) -> <Self::Link as Link<Self>>::Hosted;

    fn update(self, link: &Self::Link, built: &mut <Self::Link as Link<Self>>::Hosted) {
        *built = self.create(link);
    }
}
