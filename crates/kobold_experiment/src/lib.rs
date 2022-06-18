use std::borrow::{BorrowMut, Borrow};
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

struct Inner<S, H: Html> {
    state: RefCell<S>,
    product: RefCell<H::Built>,
    render: fn(&S, Link<S, H>) -> H,
}

pub struct Stateful<S, H: Html> {
    inner: Rc<Inner<S, H>>,
}

pub struct StatefulProduct<S, H: Html> {
    inner: Rc<Inner<S, H>>,
    js: JsValue,
}

impl<S, H: Html> Html for Stateful<S, H> {
    type Built = StatefulProduct<S, H>;

    fn build(self) -> Self::Built {
        let inner = self.inner;
        let js = inner.product.borrow_mut().js().clone();

        StatefulProduct { inner, js }
    }

    fn update(self, built: &mut Self::Built) {

    }
}

impl<S, H> Mountable for StatefulProduct<S, H>
where
    H: Html,
{
    fn js(&self) -> &JsValue {
        &self.js
    }
}

#[derive(Copy)]
struct Link<'a, S, H: Html> {
    inner: &'a Weak<Inner<S, H>>,
}

impl<S, H: Html> Clone for Link<'_, S, H> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner,
        }
    }
}

fn stateful<S, H: Html>(state: S, render: fn(&S, Link<S, H>) -> H) -> Stateful<S, H> {
    let inner = Rc::new_cyclic(move |inner| {
        let product = render(&state, Link { inner }).build();

        Inner {
            state: RefCell::new(state),
            product: RefCell::new(product),
            render,
        }
    });

    let js = inner.product.borrow().js().clone();

    Stateful { inner, js }
}

impl<S, H: Html> Link<'_, S, H> {
    fn bind(&self, fun: fn(&mut S)) -> Callback<S, H> {
        Callback {
            fun,
            link: self.clone(),
        }
    }
}

pub struct Callback<'a, S, H: Html> {
    fun: fn(&mut S),
    link: Link<'a, S, H>,
}

pub struct CallbackProduct<S> {
    fun: fn(&mut S),
    closure: Box<dyn FnMut()>,
}

impl<S, H: Html> Html for Callback<'_, S, H>
where
    S: 'static,
{
    type Built = CallbackProduct<S>;

    fn build(self) -> Self::Built {
        let weak = self.link.inner.clone();
        let fun = self.fun;

        CallbackProduct {
            fun,
            closure: Box::new(move || {
                // if let Some(rc) = weak.upgrade() {
                //     fun(&mut rc.state.borrow_mut());

                //     let html = (rc.render)(&rc.state.borrow(), Link { inner: &weak });

                //     let mut old = rc.product.borrow_mut();

                //     if let Some(old) = &mut *old {
                //         html.update(old);
                //     }
                // }
            })
        }
    }

    fn update(self, built: &mut Self::Built) {
        // TODO: update closure if fun has changed.
    }
}

impl<S> Mountable for CallbackProduct<S>
where
    S: 'static,
{
    fn js(&self) -> &JsValue {
        unimplemented!()
    }
}

struct Counter {
    n: i32,
}

impl Counter {
    pub fn render(&self) -> impl Html {
        stateful(self.n, |state, link| {
            let inc = link.bind(|n| *n += 1);
            let dec = link.bind(|n| *n -= 1);

            *state
        })
    }
}

impl Html for i32 {
    type Built = i32;

    fn build(self) -> Self::Built {
        self
    }

    fn update(self, built: &mut Self::Built) {
        *built = self;
    }
}

impl Mountable for i32 {
    fn js(&self) -> &JsValue {
        panic!()
    }
}
