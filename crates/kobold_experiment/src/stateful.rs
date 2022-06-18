use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;

use crate::{Html, Mountable};

pub trait ShouldRender {
    fn should_render(self) -> bool;
}

impl ShouldRender for () {
    fn should_render(self) -> bool {
        true
    }
}

impl ShouldRender for bool {
    fn should_render(self) -> bool {
        self
    }
}

pub struct Stateful<S, H: Html> {
    state: S,
    render: fn(&S, &Link<S, H::Product>) -> H,
}

pub fn stateful<S, H>(state: S, render: fn(&S, &Link<S, H::Product>) -> H) -> Stateful<S, H>
where
    H: Html,
{
    Stateful { state, render }
}

/// Magic wrapper for render function that allows us to store it with 'static
/// lifetime without the lifetime on return type getting in the way
#[derive(Clone, Copy)]
struct RenderFn(usize);

impl RenderFn {
    fn new<S, H: Html>(render: fn(&S, &Link<S, H::Product>) -> H) -> Self {
        RenderFn(render as usize)
    }

    unsafe fn cast<S, H: Html>(self) -> fn(&S, &Link<S, H::Product>) -> H {
        std::mem::transmute(self.0)
    }
}

struct Inner<S, P> {
    state: RefCell<S>,
    product: RefCell<P>,
    render: RenderFn,
    link: Link<S, P>,
    update: fn(RenderFn, &Link<S, P>),
}

impl<S, P> Inner<S, P> {
    fn update(&self) {
        (self.update)(self.render, &self.link)
    }
}

pub struct StatefulProduct<S, P> {
    inner: Rc<Inner<S, P>>,
    js: JsValue,
}

pub struct Link<S, P> {
    inner: Weak<Inner<S, P>>,
}

impl<S, P> Link<S, P>
where
    S: 'static,
    P: 'static,
{
    pub fn bind<F, A>(&self, f: F) -> Box<dyn FnMut() + 'static>
    where
        F: Fn(&mut S) -> A + 'static,
        A: ShouldRender,
    {
        let link = self.clone();

        Box::new(move || {
            if let Some(rc) = link.inner.upgrade() {
                if f(&mut rc.state.borrow_mut()).should_render() {
                    rc.update();
                }
            }
        })
    }
}

impl<S, P> Clone for Link<S, P> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner.clone(),
        }
    }
}

impl<S, H> Html for Stateful<S, H>
where
    S: 'static,
    H: Html,
{
    type Product = StatefulProduct<S, H::Product>;

    fn build(self) -> Self::Product {
        let inner = Rc::new_cyclic(move |inner| {
            let link = Link {
                inner: inner.clone(),
            };
            let product = (self.render)(&self.state, &link).build();

            let render = RenderFn::new(self.render);

            Inner {
                state: RefCell::new(self.state),
                product: RefCell::new(product),
                render,
                link: link.clone(),
                update: |render, link| {
                    // Safety: this is safe as long as `S` and `H` are the same types that
                    // were used to create this `RenderFn` instance.
                    let render = unsafe { render.cast::<S, H>() };

                    if let Some(inner) = link.inner.upgrade() {
                        (render)(&inner.state.borrow(), &link)
                            .update(&mut inner.product.borrow_mut());
                    }
                },
            }
        });

        let js = inner.product.borrow().js().clone();

        StatefulProduct { inner, js }
    }

    fn update(self, p: &mut Self::Product) {
        *p.inner.state.borrow_mut() = self.state;

        // p.inner.update();
        (self.render)(&p.inner.state.borrow(), &p.inner.link);
        // (p.inner.update)(p.inner.render, &p.inner.link);
    }
}

impl<S, P> Mountable for StatefulProduct<S, P>
where
    S: 'static,
    P: Mountable,
{
    fn js(&self) -> &JsValue {
        &self.js
    }
}
